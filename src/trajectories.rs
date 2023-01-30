use log::error;

pub struct TrackingTrajectories {
    pub timestep: u32,
    pub trackers: usize,
    pub trajectories: Vec<Vec<mint::Point2<i32>>>,
}

impl TrackingTrajectories {
    pub fn new(
        timestep: u32,
        trackers: usize,
        tracking_trajectories: Vec<Vec<opencv::core::Rect>>,
    ) -> Self {
        Self {
            timestep,
            trackers,
            trajectories: TrackingTrajectories::get_center_points(tracking_trajectories),
        }
    }

    pub fn get_center_points(
        tracking_trajectories: Vec<Vec<opencv::core::Rect>>,
    ) -> Vec<Vec<mint::Point2<i32>>> {
        let mut result = vec![];
        for t in tracking_trajectories {
            let mut timestamp = vec![];
            for tracking_box in t {
                timestamp.push(mint::Point2 {
                    x: tracking_box.x + tracking_box.width / 2,
                    y: tracking_box.y + tracking_box.height / 2,
                });
            }
            result.push(timestamp);
        }
        result
    }

    pub fn get_y_diff(&mut self) -> Vec<mint::Point2<i32>> {
        let mut result = vec![];
        let min_y: i32 = *self
            .trajectories
            .iter()
            .map(|a| a.first().unwrap().y)
            .collect::<Vec<i32>>()
            .iter()
            .min()
            .unwrap();
        for (idx, t) in self.trajectories.iter().enumerate() {
            match self.trackers {
                // NOTE: x has an offset of 1 frame because init frame box is not included in score
                1 => {
                    result.push(mint::Point2 {
                        x: ((idx + 1) * (self.timestep as usize)) as i32,
                        y: t.first().unwrap().y - min_y,
                    });
                }
                2 => {
                    result.push(mint::Point2 {
                        x: ((idx + 1) * (self.timestep as usize)) as i32,
                        y: t.get(0).unwrap().y - t.get(1).unwrap().y,
                    });
                }
                _ => {
                    error!("invalid tracker number");
                    break;
                }
            }
        }
        result
    }

    pub fn scale_y(
        input: Vec<mint::Point2<i32>>,
        lower: Option<i32>,
        upper: Option<i32>,
    ) -> Vec<mint::Point2<i32>> {
        let min_y = *input
            .iter()
            .map(|a| a.y)
            .collect::<Vec<i32>>()
            .iter()
            .min()
            .unwrap();

        let max_y = *input
            .iter()
            .map(|a| a.y)
            .collect::<Vec<i32>>()
            .iter()
            .max()
            .unwrap();

        input
            .iter()
            .map(|a| mint::Point2 {
                x: a.x,
                y: (((upper.unwrap_or(100) as f32) - (lower.unwrap_or(0) as f32))
                    * ((a.y as f32) - (min_y as f32))
                    / ((max_y as f32) - (min_y as f32))
                    + (lower.unwrap_or(0) as f32)) as i32,
            })
            .collect()
    }
}
