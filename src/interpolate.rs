// use log::error;

pub fn interpolate_score(
    raw_score: Vec<mint::Point2<i32>>,
    frame_step_size: u32,
) -> Option<Vec<mint::Point2<i32>>> {
    if raw_score.len() < 2 {
        return Some(raw_score);
    }

    if frame_step_size <= 1 {
        return Some(raw_score);
    }

    let x = raw_score
        .iter()
        .map(|item| item.x as f64)
        .collect::<Vec<_>>();
    let y = raw_score
        .iter()
        .map(|item| item.y as f64)
        .collect::<Vec<_>>();

    let mut spline = mentat::MonotonicCubicSpline::new(&x, &y);

    Some(
        (raw_score.first().unwrap().x..raw_score.first().unwrap().x + raw_score.len() as i32)
            .map(|x| mint::Point2 {
                x: x,
                y: spline.interpolate(x.into()) as i32,
            })
            .collect::<Vec<_>>(),
    )
}
