use crate::ffmpeg;
use crate::ffmpeg::FFmpegFrame;
use image::DynamicImage;
use log::error;
use log::info;

const FONT_NAME: &'static str = "Hack";
const FONT_SIZE: i32 = 18;

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
            FONT_NAME,
            FONT_SIZE,
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

pub async fn get_vr_viewport(
    window_name: &str,
    frame: &DynamicImage,
    video_filter_template: String,
) -> String {
    let mut pitch: i8 = -25;
    let mut yaw: i8 = 0;
    let mut fov: u8 = 90;
    let mut video_filter: String;
    loop {
        video_filter = video_filter_template
            .replace("{fov}", format!("{fov}").as_str())
            .replace("{pitch}", format!("{pitch}").as_str())
            .replace("{yaw}", format!("{yaw}").as_str());
        let mut projection = ffmpeg::transform_frame(frame, &video_filter.as_str())
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
                Some('\n') => break,
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

    video_filter
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
                FONT_NAME,
                FONT_SIZE,
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
