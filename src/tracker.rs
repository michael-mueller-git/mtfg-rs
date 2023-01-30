use crate::ffmpeg::FFmpegFrame;
use log::error;
use opencv::video::Tracker;

pub struct OpencvTracker {
    obj: opencv::core::Ptr<dyn opencv::tracking::TrackerCSRT>,
}

pub async fn track_feature(
    init_box: opencv::core::Rect,
    mut consumer: tokio::sync::mpsc::Receiver<FFmpegFrame<'_>>,
    producer: tokio::sync::mpsc::Sender<opencv::core::Rect>,
) {
    let tracker_param: opencv::tracking::TrackerCSRT_Params =
        opencv::tracking::TrackerCSRT_Params::default().unwrap();
    let mut tracker: OpencvTracker = OpencvTracker {
        obj: <dyn opencv::tracking::TrackerCSRT>::create(&tracker_param).unwrap(),
    };

    let Some(mut init_frame) = consumer.recv().await else {
        error!("init frame missing");
        return;
    };

    let mut exit = false;
    let mut opencv_frame = init_frame.get_opencv_frame();
    opencv_frame.with_mut(|frame| {
        if tracker.obj.init(frame.mat, init_box).is_err() {
            error!("tracker setup failed");
            exit = true;
        }
    });

    if exit {
        return;
    }

    let mut bounding_box = init_box;
    while let Some(mut frame) = consumer.recv().await {
        let mut opencv_frame = frame.get_opencv_frame();
        opencv_frame.with_mut(|frame| {
            if tracker.obj.update(frame.mat, &mut bounding_box).is_err() {
                error!("tracking lost");
                exit = true;
            }
        });

        if exit {
            break;
        }

        if producer.send(bounding_box).await.is_err() {
            error!("tracker: error adding box to process queue");
            break;
        }
    }
}


