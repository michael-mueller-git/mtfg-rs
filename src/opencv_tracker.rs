use crate::ffmpeg_stream::FFmpegFrame;
use log::error;
use log::info;
use opencv::video::Tracker;

pub struct OpencvTracker {
    obj: opencv::core::Ptr<dyn opencv::tracking::TrackerCSRT>,
}

// The following hack is required for non blocking tokio tasks:
// unsafe impl Send for OpencvTracker {}
// unsafe impl Sync for OpencvTracker {}

pub async fn get_rois(
    boxes: usize,
    window_name: &str,
    frame: &mut FFmpegFrame,
) -> Vec<opencv::core::Rect> {
    let mut input: Vec<opencv::core::Rect> = vec![];
    let mut frame = frame.clone();
    let mut opencv_frame = frame.get_opencv_frame();

    opencv::highgui::named_window(
        window_name,
        opencv::highgui::WINDOW_AUTOSIZE | opencv::highgui::WINDOW_GUI_NORMAL,
    )
    .unwrap();

    while input.len() < boxes {
        match opencv::highgui::select_roi_for_window(window_name, &opencv_frame, true, false) {
            Ok(result) => {
                if result.x != 0 && result.y != 0 {
                    opencv::imgproc::rectangle(
                        &mut opencv_frame,
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

    input
}

pub async fn track_feature(
    init_box: opencv::core::Rect,
    mut consumer: tokio::sync::mpsc::Receiver<FFmpegFrame>,
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

    let opencv_frame = init_frame.get_opencv_frame();
    if tracker.obj.init(&opencv_frame, init_box).is_err() {
        error!("tracker setup failed");
        return;
    }

    let mut bounding_box = init_box;
    while let Some(mut frame) = consumer.recv().await {
        let opencv_frame = frame.get_opencv_frame();
        if tracker
            .obj
            .update(&opencv_frame, &mut bounding_box)
            .is_err()
        {
            error!("tracking lost");
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
    frame: &mut FFmpegFrame,
    boxes: &Vec<opencv::core::Rect>,
    text: &str,
) -> bool {
    let mut opencv_frame = frame.get_opencv_frame();
    for tracking_box in boxes {
        opencv::imgproc::rectangle(
            &mut opencv_frame,
            *tracking_box,
            opencv::core::Scalar::new(0f64, -1f64, -1f64, -1f64),
            2,
            8,
            0,
        )
        .unwrap();
    }

    if !text.is_empty() {
        opencv::highgui::add_text_with_font(
            &opencv_frame,
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

    opencv::highgui::imshow(window_name, &opencv_frame).unwrap();

    let key = opencv::highgui::wait_key(1).unwrap();
    if key == 'q' as i32 {
        info!("stop requested by user");
        true
    } else {
        false
    }
}
