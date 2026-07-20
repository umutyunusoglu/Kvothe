use num::Float;

pub fn remove_dc_offset<T: Float>(signal: &[T]) -> Vec<T> {
    let mean = signal.iter().fold(T::zero(), |acc, &x| acc + x) / T::from(signal.len()).unwrap();

    signal.iter().map(|&x| x - mean).collect()
}

pub fn apply_hann_window<T: Float>(signal: &[T]) -> Vec<T> {
    let n = signal.len();
    if n < 2 {
        return signal.to_vec();
    }

    let two_pi = T::from(2.0 * std::f64::consts::PI).unwrap();
    let denom = T::from(n - 1).unwrap();

    signal
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let phase = two_pi * T::from(i).unwrap() / denom;
            let w = T::from(0.5).unwrap() * (T::one() - phase.cos());
            x * w
        })
        .collect()
}

pub fn calculate_threshold<T: Float>(magnitudes: &[T]) -> T {
    let mean =
        magnitudes.iter().fold(T::zero(), |acc, &x| acc + x) / T::from(magnitudes.len()).unwrap();
    let std = (magnitudes
        .iter()
        .fold(T::zero(), |acc, &x| acc + (x - mean).powi(2))
        / T::from(magnitudes.len()).unwrap())
    .sqrt();

    mean + T::from(2.0).unwrap() * std
}
