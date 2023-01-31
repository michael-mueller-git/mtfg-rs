

pub fn rdp(score: Vec<mint::Point2<i32>>, epsilon: f64) -> Vec<mint::Point2<i32>> {
    let mut keep = (0..score.len()).collect();

    if epsilon > 0.01 {
        keep = ramer_douglas_peucker::rdp(&score, epsilon);
    }

    score
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| keep.contains(idx))
        .map(|(_, val)| val)
        .collect::<Vec<_>>()
}
