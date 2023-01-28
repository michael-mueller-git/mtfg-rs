use crate::ffmpeg_stream::FFmpegFrame;
use log::error;
use log::info;
use opencv::video::Tracker;

pub struct OpencvTracker {
    obj: opencv::core::Ptr<dyn opencv::tracking::TrackerCSRT>,
}

pub async fn get_rois(
    boxes: usize,
    window_name: &str,
    frame: &mut FFmpegFrame<'_>,
) -> Vec<opencv::core::Rect> {
    let mut input: Vec<opencv::core::Rect> = vec![];
    let mut frame = frame.clone();
    let mut opencv_frame = frame.get_opencv_frame();

    opencv::highgui::named_window(
        window_name,
        opencv::highgui::WINDOW_AUTOSIZE | opencv::highgui::WINDOW_GUI_NORMAL,
    )
    .unwrap();

    opencv_frame.with_mut(|frame| {
        opencv::highgui::add_text_with_font(
            frame.mat,
            "Select Tracking Features",
            opencv::core::Point::new(5, 30),
            "Hack",
            20,
            opencv::core::Scalar::new(0f64, -1f64, -1f64, -1f64),
            0, /* opencv::highgui::QtFontWeights::QT_FONT_NORMAL */
            0, /* opencv::highgui::QtFontStyles::QT_STYLE_NORMAL */
            0,
        )
        .unwrap();
        while input.len() < boxes {
            match opencv::highgui::select_roi_for_window(window_name, frame.mat, true, false) {
                Ok(result) => {
                    if result.x != 0 && result.y != 0 {
                        opencv::imgproc::rectangle(
                            &mut frame.mat,
                            result,
                            opencv::core::Scalar::new(0f64, -1f64, -1f64, -1f64),
                            2,
                            8,
                            0,
                        )
                        .unwrap();
                        input.push(result);
                    } else {
                        error!("Invalid Input");
                    }
                }
                Err(_) => {
                    error!("Input Error");
                }
            }
        }
    });

    input
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

pub async fn preview_tracking_boxes(
    window_name: &str,
    frame: &mut FFmpegFrame<'_>,
    boxes: &Vec<opencv::core::Rect>,
    text: &str,
) -> bool {
    let mut opencv_frame = frame.get_opencv_frame();
    for tracking_box in boxes {
        opencv_frame.with_mut(|frame| {
            opencv::imgproc::rectangle(
                &mut frame.mat,
                *tracking_box,
                opencv::core::Scalar::new(0f64, -1f64, -1f64, -1f64),
                2,
                8,
                0,
            )
            .unwrap()
        });
    }

    opencv_frame.with_mut(|frame| {
        if !text.is_empty() {
            opencv::highgui::add_text_with_font(
                frame.mat,
                text,
                opencv::core::Point::new(5, 30),
                "Hack",
                20,
                opencv::core::Scalar::new(0f64, -1f64, -1f64, -1f64),
                0, /* opencv::highgui::QtFontWeights::QT_FONT_NORMAL */
                0, /* opencv::highgui::QtFontStyles::QT_STYLE_NORMAL */
                0,
            )
            .unwrap();
        }
    });

    opencv_frame.with_mut(|frame| opencv::highgui::imshow(window_name, frame.mat).unwrap());

    let key = opencv::highgui::wait_key(1).unwrap();
    if key == 'q' as i32 {
        info!("stop requested by user");
        true
    } else {
        false
    }
}
