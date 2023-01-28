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
    let args = args::parse_args();
    logging::setup_logging();

    if args.persons > 2 {
        error!("invalid args.person");
        return;
    }

    let window_name = "mtfg-rs";
    let channel_capacity = 64;


    let opencv_frame = ffmpeg_stream::get_single_frame2(&args.input, 0, ffmpeg_stream::Dimensions{width: 1280, height: 720}).await;

    // let opencv_frame2 = opencv_frame.unwrap();
    // let mut opencv_frame3 = opencv_frame2.unwrap();
    // let opencv_frame4 = *opencv_frame3.get_opencv_frame();

    // let mut opencv_frame3 = opencv_frame.unwrap().unwrap();
    // let opencv_frame4 = *opencv_frame3.get_opencv_frame();

    let mut opencv_frame4 = *opencv_frame.unwrap().unwrap().get_opencv_frame();

    std::thread::sleep(std::time::Duration::from_millis(1000));
    opencv::imgcodecs::imwrite("./test.png", &opencv_frame4, &opencv::core::Vector::default()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(1000));
    // opencv_frame3.image.save("test_image.png").unwrap();


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
            .block_on(ffmpeg_stream::spawn_ffmpeg_frame_reader(args, frame_sender));
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
