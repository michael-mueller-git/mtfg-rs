use log::error;

pub fn interpolate_score(raw_score: Vec<mint::Point2<i32>>, frame_step_size: u32) -> Option<Vec<mint::Point2<i32>>> {
    let raw_score_vec = raw_score
        .iter()
        .map(|item| (item.x as f64, item.y as f64))
        .collect::<Vec<_>>();

    if frame_step_size <= 1 {
        Some(raw_score)
    } else {
        let opts = cubic_spline::SplineOpts::new().tension(0.5); // TODO hyperparam

        let Ok(points) = cubic_spline::Points::try_from(&raw_score_vec) else {
            error!("Create Interpolation failed");
            return None;
        };

        let calculated_points = points
            .calc_spline(&opts.num_of_segments(frame_step_size - 1))
            .unwrap();

        Some(calculated_points
            .into_inner()
            .iter()
            .map(|item| mint::Point2 {
                x: item.x as i32,
                y: item.y as i32,
            })
            .collect())
    }
}
