use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FunscriptAction {
    pos: u8,
    at: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FunscriptContent {
    version: String,
    inverted: bool,
    range: u8,
    fps: Option<f32>,
    actions: Vec<FunscriptAction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Funscript {
    pub video_fps: f32,
    pub start_time_in_ms: f32,
    pub content: FunscriptContent,
}

impl Funscript {
    pub fn new(video_fps: f32, start_time_in_ms: f32, score: Vec<&mint::Point2<i32>>) -> Self {
        Self {
            video_fps: video_fps,
            start_time_in_ms: start_time_in_ms,
            content: Funscript::to_funscript_content(score, video_fps, start_time_in_ms),
        }
    }

    fn to_funscript_content(
        score: Vec<&mint::Point2<i32>>,
        video_fps: f32,
        start_time_in_ms: f32,
    ) -> FunscriptContent {
        let frame_time_in_ms = 1000.0 / video_fps;

        FunscriptContent {
            version: "1.0".to_string(),
            inverted: false,
            range: 90,
            fps: Some(video_fps),
            actions: score
                .iter()
                .map(|a| FunscriptAction {
                    pos: a.y as u8,
                    at: (start_time_in_ms + frame_time_in_ms * (a.x as f32)) as u32,
                })
                .collect(),
        }
    }

    pub fn save(self: &mut Self, file_path: &str) {
        let serialized_funscript = serde_json::to_string(&self.content).unwrap();
        info!("save funscript to {file_path}");
        std::fs::write(file_path, serialized_funscript)
            .expect(format!("Unable to write funscript: {file_path}").as_str());
    }
}
