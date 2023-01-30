use clap::Parser;
use log::error;

#[derive(Parser)]
#[clap(
    name = "mtfg-rs",
    about = "Motion Tracking Funscript Generator",
    version,
    author
)]
pub struct Args {
    /// Path to Video File
    #[clap(short = 'i', long = "input")]
    pub input: String,

    /// Output Path
    #[clap(short = 'o', long = "output")]
    pub output: String,

    /// Start time in milliseconds
    #[clap(short = 's', long = "start")]
    pub start_time: f32,

    /// End time in milliseconds
    #[clap(long = "end")]
    pub end_time: Option<f32>,

    /// Number of moving persons
    #[clap(long = "persons", default_value = "1")]
    pub persons: u8,

    /// Frame Step Size
    #[clap(long = "step", default_value = "2")]
    pub frame_step_size: u32,

    /// Preview frame step size
    #[clap(long = "preview", default_value = "2")]
    pub preview_frames: u32,

    /// Video Filter with output 'w=\d:h=\d' parameter
    #[clap(
        long = "filter",
        default_value = "v360=input=he:in_stereo=sbs:pitch={pitch}:yaw={yaw}:roll=0:output=flat:d_fov={fov}:w=800:h=800"
    )]
    pub video_filter: String,

    /// epsilon value for Ramer–Douglas–Peucker algorithm
    #[clap(long = "epsilon")]
    pub epsilon: f64,
}

impl Clone for Args {
    fn clone(&self) -> Args {
        Args {
            input: self.input.clone(),
            output: self.output.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            frame_step_size: self.frame_step_size,
            preview_frames: self.preview_frames,
            video_filter: self.video_filter.clone(),
            persons: self.persons,
            epsilon: self.epsilon
        }
    }
}

pub fn parse_args() -> Option<Args> {
    let result = Args::parse();

    if result.frame_step_size < 1 {
        error!("Invalid step value");
        return None;
    }

    if result.preview_frames < 1 {
        error!("Invalid preview value");
        return None;
    }

    if result.persons < 1 || result.persons > 2 {
        error!("Invalid persons value");
        return None;
    }

    if result.epsilon < 0.0 {
        error!("Invalid epsilon value");
        return None;
    }

    Some(result)
}
