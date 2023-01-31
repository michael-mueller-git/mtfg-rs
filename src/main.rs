mod args;
mod ffmpeg;
mod funscript;
mod interpolate;
mod logging;
mod simplify;
mod tracker;
mod trajectories;
mod ui;

use log::error;

const WINDOW_NAME: &'static str = "mtfg-rs";
const CHANNEL_CAPACITY: usize = 64;

#[tokio::main(worker_threads = 6)]
async fn main() {
    let Some(mut args) = args::parse_args() else {
        return;
    };

    logging::setup_logging();

    let Ok(video_fps) = ffmpeg::get_video_fps(args.input.as_str()) else {
        error!("Could not determine video fps");
        return;
    };

    let preview_frame = ffmpeg::get_single_frame(&args.input.as_str(), args.start_time as u32)
        .await
        .unwrap()
        .unwrap();

    args.video_filter =
        ui::get_vr_viewport(WINDOW_NAME, &*preview_frame.image, args.video_filter).await;

    let mut frame_sender = vec![];
    let mut frame_receiver = vec![];
    let mut tracking_sender = vec![];
    let mut tracking_receiver = vec![];

    for _ in 0..args.persons {
        let (frame_tx, frame_rx) =
            tokio::sync::mpsc::channel::<ffmpeg::FFmpegFrame>(CHANNEL_CAPACITY);
        frame_sender.push(frame_tx);
        frame_receiver.push(frame_rx);

        let (tracking_tx, tracking_rx) =
            tokio::sync::mpsc::channel::<opencv::core::Rect>(CHANNEL_CAPACITY);
        tracking_sender.push(tracking_tx);
        tracking_receiver.push(tracking_rx);
    }

    let (frame_tx, mut frame_rx) =
        tokio::sync::mpsc::channel::<ffmpeg::FFmpegFrame>(CHANNEL_CAPACITY);
    frame_sender.push(frame_tx); // preview

    let ffmpeg_args = args.clone();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current()
            .block_on(ffmpeg::ffmpeg_stream_reader(ffmpeg_args, frame_sender));
    });

    let Some(mut frame) = frame_rx.recv().await else {
        error!("Extract first frame failed");
        return;
    };

    let mut tracking_boxes = ui::get_rois(args.persons as usize, WINDOW_NAME, &mut frame).await;

    while let Some(b) = tracking_boxes.pop() {
        if let Some(r) = frame_receiver.pop() {
            if let Some(p) = tracking_sender.pop() {
                tokio::task::spawn_blocking(move || {
                    tokio::runtime::Handle::current().block_on(tracker::track_feature(b, r, p));
                });
            } else {
                error!("Not enough sender obj available");
            }
        } else {
            error!("Not enough receiver obj available");
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
                error!("Tracking box missing");
                continue;
            };
            result.push(tracking_box);
        }

        let mut stop = false;

        if ((frame_counter - 1) % args.preview_frames) == 0 {
            let fps = (args.frame_step_size * frame_counter * 1000) as u128
                / start_time.elapsed().as_millis();

            stop = ui::preview_tracking_boxes(
                WINDOW_NAME,
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
        args.frame_step_size,
        args.persons as usize,
        tracking_trajectories,
    );

    let raw_score = trajectories::TrackingTrajectories::scale_y(
        tracking_result.get_y_diff(),
        Some(100),
        Some(0),
    );

    let Some(interpolated_score) = interpolate::interpolate_score(raw_score, args.frame_step_size) else {
        error!("Create funscript FAILED");
        return;
    };

    let score = simplify::rdp(interpolated_score, args.epsilon);

    let mut funscript = funscript::Funscript::new(video_fps, args.start_time, score);
    funscript.save(args.output.as_str());
}
