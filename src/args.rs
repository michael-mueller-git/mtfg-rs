use clap::Parser;

#[derive(Clone, Parser)]
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
    #[clap(short = 'e', long = "end")]
    pub end_time: Option<f32>,

    /// Skip Frames
    #[clap(short = 'k', long = "skip", default_value = "1")]
    pub skip_frames: u32,

    /// Preview only given frames
    #[clap(long = "preview", default_value = "1")]
    pub preview_frames: u32,

    /// Video Filter with output 'w=\d:h=\d' parameter
    #[clap(short = 'f', long = "filter", default_value = "v360=input=he:in_stereo=sbs:pitch={pitch}:yaw={yaw}:roll=0:output=flat:d_fov=90:w=800:h=800")]
    pub video_filter: String,

    /// Number of moving persons
    #[clap(short = 'p', long = "persons", default_value = "1")]
    pub persons: u8,

    /// epsilon value for Ramer–Douglas–Peucker algorithm
    #[clap(short = 'a', long = "epsilon")]
    pub epsilon: f64,
}

pub fn parse_args() -> Args {
    Args::parse()
}
