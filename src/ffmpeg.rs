use bytes::BytesMut;
use fortify::*;
use fraction::ToPrimitive;
use futures_util::StreamExt;
use image::DynamicImage;
use image::ImageBuffer;
use image::Rgb;
use log::error;
use log::info;
use log::warn;
use std::boxed::Box;
use std::io;
use std::io::{BufRead, BufReader};
use std::marker::PhantomData;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio_util::codec::Decoder;
use tokio_util::codec::FramedRead;

use crate::args;

pub type Bgr = Rgb<u8>;
pub type FrameBuffer = ImageBuffer<Bgr, Vec<u8>>;

#[derive(Copy, Clone)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

pub struct VideoFrame {
    capacity: usize,
}

impl VideoFrame {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            capacity: (width * height * 3) as usize,
        }
    }
}

impl Decoder for VideoFrame {
    type Error = io::Error;
    type Item = BytesMut;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.capacity() < self.capacity {
            src.reserve(self.capacity)
        }
        if src.len() >= self.capacity {
            Ok(Some(src.split_to(self.capacity)))
        } else {
            Ok(None)
        }
    }
}

#[derive(Lower)]
pub struct OpencvMatWithLifetime<'a> {
    pub mat: &'a mut opencv::core::Mat,
}

#[derive(Clone)]
pub struct FFmpegFrame<'a> {
    pub image: Arc<DynamicImage>,
    image_lifetime: PhantomData<&'a DynamicImage>,
}

impl<'a> FFmpegFrame<'a> {
    pub fn new(frame_buffer: FrameBuffer) -> Self {
        Self {
            // NOTE: We store bgr image in rgb buffer!
            image: Arc::new(DynamicImage::ImageRgb8(frame_buffer)),
            image_lifetime: PhantomData,
        }
    }

    pub fn get_opencv_frame(&mut self) -> Fortify<OpencvMatWithLifetime> {
        unsafe {
            fortify! {
                let ensure_lifetime = &self.image;
                let mut m = opencv::prelude::Mat::new_rows_cols_with_data(
                    ensure_lifetime.height() as i32,
                    ensure_lifetime.width() as i32,
                    opencv::core::CV_8UC3,
                    ensure_lifetime.as_rgb8().unwrap().as_ptr() as *mut _,
                    opencv::core::Mat_AUTO_STEP,
                )
                .unwrap();
                yield OpencvMatWithLifetime {
                    mat: &mut m,
                };
            }
        }
    }
}

pub fn get_video_fps(video_path: &str) -> Result<f32, Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=r_frame_rate",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let fps = stdout_reader
        .lines()
        .next()
        .unwrap()?
        .parse::<fraction::Fraction>()?;

    cmd.wait()?;

    match fps.to_f32() {
        Some(val) => Ok(val),
        None => {
            warn!("could not determine fps of video {video_path}");
            Ok(30.0)
        }
    }
}

pub fn get_video_dimensions(video_path: &str) -> Result<Dimensions, Box<dyn std::error::Error>> {
    let mut cmd = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=s=x:p=0",
            video_path,
        ])
        .stdout(Stdio::piped())
        .spawn()?;

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let resolution = stdout_reader.lines().next().unwrap()?;

    let mut video_dimensions: Dimensions = Dimensions {
        width: 0,
        height: 0,
    };

    let re = regex::Regex::new(r"(\d+)x(\d+)").unwrap();
    for cap in re.captures_iter(&resolution) {
        let w = cap[1].parse::<u32>()?;
        let h = cap[2].parse::<u32>()?;
        video_dimensions = Dimensions::new(w, h);
    }

    cmd.wait()?;

    info!(
        "Video resolution: {}x{}",
        video_dimensions.width, video_dimensions.height
    );

    Ok(video_dimensions)
}

pub async fn spawn_ffmpeg_frame_reader(
    args: Vec<&str>,
    frame_dimensions: Dimensions,
    input: Option<&[u8]>,
) -> FramedRead<tokio::process::ChildStdout, VideoFrame> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(args);

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = cmd.spawn().expect("failed to spawn ffmpeg");

    let stdout = child
        .stdout
        .take()
        .expect("ffmpeg process did not have a handle to stdout");

    let mut stdin = child
        .stdin
        .take()
        .expect("ffmpeg process did not have a handle to stdin");

    tokio::spawn(async move {
        let _ = child
            .wait()
            .await
            .expect("ffmpeg process encountered an error");
    });

    if let Some(input) = input {
        if stdin.write_all(input).await.is_err() {
            error!("ffmpeg stdin failed");
        }
    }

    drop(stdin);

    FramedRead::new(
        stdout,
        VideoFrame::new(frame_dimensions.width, frame_dimensions.height),
    )
}

pub async fn get_single_frame(
    video_path: &str,
    timestamp_in_ms: u32,
) -> Result<Option<FFmpegFrame>, Box<dyn std::error::Error>> {
    let Ok(video_dimensions) = get_video_dimensions(video_path) else {
        error!("Invalid video dimensions");
        return Ok(None);
    };

    let mut reader = spawn_ffmpeg_frame_reader(
        vec![
            "-hide_banner",
            "-loglevel",
            "warning",
            "-ss",
            millisec_to_timestamp(timestamp_in_ms).as_str(),
            "-hwaccel",
            "auto",
            "-i",
            video_path,
            "-vframes",
            "1",
            "-f",
            "image2pipe",
            "-pix_fmt",
            "bgr24",
            "-fps_mode",
            "passthrough",
            "-vcodec",
            "rawvideo",
            "-an",
            "-sn",
            "-",
        ],
        video_dimensions,
        None,
    )
    .await;

    if let Some(Ok(bytes_mut_buffer)) = reader.next().await {
        let frame_buffer: FrameBuffer = FrameBuffer::from_raw(
            video_dimensions.width,
            video_dimensions.height,
            bytes_mut_buffer.to_vec(),
        )
        .expect("ffmpeg: parse frame error");
        Ok(Some(FFmpegFrame::new(frame_buffer)))
    } else {
        warn!("No data in ffmpeg output buffer");
        Ok(None)
    }
}

pub async fn transform_frame<'a>(
    frame: &DynamicImage,
    video_filter: &str,
) -> Result<Option<FFmpegFrame<'a>>, Box<dyn std::error::Error>> {
    let input_dimensions = Dimensions {
        width: frame.width(),
        height: frame.height() as u32,
    };
    // TODO ensure input image is min 1,5x larger than output (use image resize)
    let Some(output_dimensions) = get_dimension_from_video_filter(video_filter) else {
        error!("Failed to parse video filter dimensions");
        return Ok(None);
    };

    let mut reader = spawn_ffmpeg_frame_reader(
        vec![
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "rawvideo",
            "-vcodec",
            "rawvideo",
            "-s",
            format!("{}x{}", input_dimensions.width, input_dimensions.height).as_str(),
            "-pix_fmt",
            "bgr24",
            "-i",
            "-",
            "-f",
            "image2pipe",
            "-pix_fmt",
            "bgr24",
            "-vsync",
            "passthrough",
            "-vcodec",
            "rawvideo",
            "-an",
            "-sn",
            "-vf",
            video_filter,
            "-",
        ],
        output_dimensions,
        Some(frame.as_rgb8().unwrap()),
    )
    .await;

    if let Some(Ok(bytes_mut_buffer)) = reader.next().await {
        let frame_buffer: FrameBuffer = FrameBuffer::from_raw(
            output_dimensions.width,
            output_dimensions.height,
            bytes_mut_buffer.to_vec(),
        )
        .expect("ffmpeg: parse frame error");
        Ok(Some(FFmpegFrame::new(frame_buffer)))
    } else {
        Ok(None)
    }
}

fn millisec_to_timestamp(val: u32) -> String {
    let seconds = (val / 1000) % 60;
    let minutes = (val / (1000 * 60)) % 60;
    let hours = (val / (1000 * 60 * 60)) % 24;
    let millis = val % 1000;
    format!("{hours:0>2}:{minutes:0>2}:{seconds:0>2}.{millis:0>3}")
}

fn get_dimension_from_video_filter(video_filter: &str) -> Option<Dimensions> {
    let mut video_dimensions: Option<Dimensions> = None;
    let re = regex::Regex::new(r"w=(\d+):h=(\d+)").unwrap();
    for cap in re.captures_iter(video_filter) {
        let w = cap[1]
            .parse::<u32>()
            .expect("failed to parse video filter width");
        let h = cap[2]
            .parse::<u32>()
            .expect("failed to parse video filter height");
        video_dimensions = Some(Dimensions::new(w, h));
    }

    if video_dimensions.is_none() {
        let re = regex::Regex::new(r"scale=(\d+):(\d+)").unwrap();
        for cap in re.captures_iter(video_filter) {
            let w = cap[1]
                .parse::<u32>()
                .expect("failed to parse video filter width");
            let h = cap[2]
                .parse::<u32>()
                .expect("failed to parse video filter height");
            video_dimensions = Some(Dimensions::new(w, h));
        }
    }
    video_dimensions
}

pub async fn ffmpeg_stream_reader(
    args: args::Args,
    producers: Vec<tokio::sync::mpsc::Sender<FFmpegFrame<'_>>>,
) {
    let fps = get_video_fps(args.input.as_str()).unwrap_or(30.0);
    let stop_frame_count = match args.end_time {
        Some(val) => {
            let start_frame = (args.start_time * fps / 1000.0) as u32;
            let stop_frame = (val * fps / 1000.0) as u32;
            if stop_frame > start_frame {
                stop_frame - start_frame
            } else {
                0
            }
        }
        None => 0,
    };

    let Some(video_dimensions) = get_dimension_from_video_filter(args.video_filter.as_str()) else {
        error!("Failed to parse video filter dimensions");
        return;
    };

    let mut reader = spawn_ffmpeg_frame_reader(
        vec![
            "-hide_banner",
            "-loglevel",
            "warning",
            "-hwaccel",
            "auto",
            "-ss",
            millisec_to_timestamp(args.start_time as u32).as_str(),
            "-i",
            args.input.as_str(),
            "-f",
            "image2pipe",
            "-pix_fmt",
            "bgr24",
            "-vcodec",
            "rawvideo",
            "-an",
            "-sn",
            "-vf",
            args.video_filter.as_str(),
            "-",
        ],
        video_dimensions,
        None,
    )
    .await;

    info!("start ffmpeg");

    let mut frame_number = 0;
    while let Some(Ok(bytes_mut_buffer)) = reader.next().await {
        let frame_buffer: FrameBuffer = FrameBuffer::from_raw(
            video_dimensions.width,
            video_dimensions.height,
            bytes_mut_buffer.to_vec(),
        )
        .expect("ffmpeg: parse frame error");
        frame_number += 1;

        if ((frame_number - 1) % args.frame_step_size) != 0 {
            continue;
        }

        if stop_frame_count > 0 && stop_frame_count < frame_number {
            info!("ffmpeg: reach specified end frame");
            break;
        }

        let mut should_exit = false;
        let ffmpeg_frame = FFmpegFrame::new(frame_buffer);
        for producer in &producers {
            if producer.send(ffmpeg_frame.clone()).await.is_err() {
                error!("ffmpeg: error adding frame to process queue");
                should_exit = true;
                break;
            }
        }

        if should_exit {
            break;
        }
    }

    info!("stop ffmpeg");
}
