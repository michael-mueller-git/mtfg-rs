mod args;
mod ffmpeg_stream;
mod funscript;
mod logging;
mod opencv_tracker;
mod trajectories;

use log::error;
use log::info;

#[tokio::main(worker_threads = 6)]
async fn main() {
    let mut args = args::parse_args();
    logging::setup_logging();

    if args.persons > 2 {
        error!("invalid args.person");
        return;
    }

    let window_name = "mtfg-rs";
    let channel_capacity = 64;

    let video_path = args.input.clone();
    let start_frame = ffmpeg_stream::get_single_frame(&video_path.as_str(), args.start_time as u32)
        .await
        .unwrap()
        .unwrap();

    let mut pitch: i8 = -25;
    let mut yaw: i8 = 0;
    let mut fov: u8 = 90;
    let mut video_filter: String;
    loop {
        video_filter = args
            .video_filter
            .replace("{fov}", format!("{fov}").as_str())
            .replace("{pitch}", format!("{pitch}").as_str())
            .replace("{yaw}", format!("{yaw}").as_str());
        let mut projection =
            ffmpeg_stream::transform_frame(&*start_frame.image, &video_filter.as_str())
                .await
                .unwrap()
                .unwrap();
        projection
            .get_opencv_frame()
            .with_mut(|frame| opencv::highgui::imshow(window_name, frame.mat).unwrap());
        let key = opencv::highgui::wait_key(1).unwrap();
        if key > 0 {
            match char::from_u32(key.try_into().unwrap()) {
                Some('q') => break,
                Some(' ') => break,
                Some('w') => pitch += 5,
                Some('s') => pitch -= 5,
                Some('a') => yaw -= 5,
                Some('d') => yaw += 5,
                Some('+') => fov -= 5,
                Some('-') => fov += 5,
                _ => {}
            };
        }
    }

    args.video_filter = video_filter;

    let mut frame_sender = vec![];
    let mut frame_receiver = vec![];
    let mut tracking_sender = vec![];
    let mut tracking_receiver = vec![];

    for _ in 0..args.persons {
        let (frame_tx, frame_rx) =
            tokio::sync::mpsc::channel::<ffmpeg_stream::FFmpegFrame>(channel_capacity);
        frame_sender.push(frame_tx);
        frame_receiver.push(frame_rx);

        let (tracking_tx, tracking_rx) =
            tokio::sync::mpsc::channel::<opencv::core::Rect>(channel_capacity);
        tracking_sender.push(tracking_tx);
        tracking_receiver.push(tracking_rx);
    }

    let (frame_tx, mut frame_rx) =
        tokio::sync::mpsc::channel::<ffmpeg_stream::FFmpegFrame>(channel_capacity);
    frame_sender.push(frame_tx);

    // TODO: handle moved values better
    let skip_frames = args.skip_frames;
    let number_of_tracking_boxes = args.persons as usize;
    let epsilon = args.epsilon;
    let preview_frames = args.preview_frames;
    let video_start_time_in_ms = args.start_time;
    let output = args.output.clone();

    let Ok(video_fps) = ffmpeg_stream::get_video_fps(args.input.as_str()) else {
        error!("could not determine video fps");
        return;
    };

    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current()
            .block_on(ffmpeg_stream::ffmpeg_stream_reader(args, frame_sender));
    });

    let Some(mut frame) = frame_rx.recv().await else {
        error!("extract first frame failed");
        return;
    };

    let mut tracking_boxes =
        opencv_tracker::get_rois(number_of_tracking_boxes, window_name, &mut frame).await;

    while let Some(b) = tracking_boxes.pop() {
        if let Some(r) = frame_receiver.pop() {
            if let Some(p) = tracking_sender.pop() {
                tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current()
                        .block_on(opencv_tracker::track_feature(b, r, p));
                });
            } else {
                error!("not enough sender obj available");
            }
        } else {
            error!("not enough receiver obj available");
        }
    }

    let start_time = std::time::Instant::now();
    let mut frame_counter = 0;
    let mut tracking_trajectories = vec![];
    while let Some(mut frame) = frame_rx.recv().await {
        frame_counter += 1;

        let mut result = vec![];
        for item in tracking_receiver.iter_mut() {
            let Some(tracking_box) = item.recv().await else {
                error!("tracking box missing");
                continue;
            };
            result.push(tracking_box);
        }

        let mut stop = false;

        if ((frame_counter - 1) % (preview_frames + 1)) == 0 {
            let fps = ((skip_frames + 1) * frame_counter * 1000) as u128
                / start_time.elapsed().as_millis();

            stop = opencv_tracker::preview_tracking_boxes(
                window_name,
                &mut frame,
                &result,
                format!("{fps} fps").as_str(),
            )
            .await;
        }

        tracking_trajectories.push(result);

        if stop {
            break;
        }
    }

    let mut tracking_result = trajectories::TrackingTrajectories::new(
        skip_frames + 1,
        number_of_tracking_boxes,
        tracking_trajectories,
    );

    let raw_score = trajectories::TrackingTrajectories::scale_y(
        tracking_result.get_y_diff(),
        Some(100),
        Some(0),
    );

    let keep = ramer_douglas_peucker::rdp(&raw_score, epsilon);

    let score = raw_score
        .iter()
        .enumerate()
        .filter(|(idx, _)| keep.contains(idx))
        .map(|(_, val)| val)
        .collect::<Vec<_>>();

    let mut funscript = funscript::Funscript::new(video_fps, video_start_time_in_ms, score);
    funscript.save(output.as_str());
    info!("program exit");
}
