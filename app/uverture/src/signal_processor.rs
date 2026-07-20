use num::Float;
use num_complex::Complex;
use rustfft::{FftNum, FftPlanner};
use std::sync::Mutex;
mod error;
mod math;
use error::SignalError;
pub struct SignalProcessor<T: FftNum + Float> {
    sample_rate: usize,
    fft_size: usize,
    planner: Mutex<FftPlanner<T>>,
}

impl<T: FftNum + Float> SignalProcessor<T> {
    pub fn new(sample_rate: usize, fft_size: usize) -> Result<SignalProcessor<T>, SignalError> {
        if sample_rate == 0 {
            return Err(SignalError::InvalidSampleRate);
        }
        if fft_size == 0 {
            return Err(SignalError::InvalidNumberOfSamples);
        }
        Ok(SignalProcessor {
            sample_rate,
            fft_size,
            planner: Mutex::new(FftPlanner::new()),
        })
    }

    pub fn fft(&self, signal: &[T]) -> Result<Vec<Complex<T>>, SignalError> {
        if signal.len() != self.fft_size {
            return Err(SignalError::InvalidNumberOfSamples);
        }

        let fft = self
            .planner
            .lock()
            .map_err(|_| SignalError::ProcessorLockError)?
            .plan_fft_forward(self.fft_size);

        let mut buffer: Vec<Complex<T>> = signal
            .iter()
            .map(|&s| Complex {
                re: s,
                im: T::zero(),
            })
            .collect();

        fft.process(&mut buffer);
        Ok(buffer)
    }

    pub fn magnitude_spectrum(&self, spectrum: &[Complex<T>]) -> Result<Vec<T>, SignalError> {
        if spectrum.is_empty() {
            return Err(SignalError::EmptySpectrum);
        }

        Ok(spectrum[..spectrum.len() / 2]
            .iter()
            .map(|c| c.re.hypot(c.im))
            .collect())
    }

    pub fn peak_frequencies(&self, signal: &[T]) -> Result<Vec<T>, SignalError> {
        if signal.is_empty() {
            return Err(SignalError::EmptySignal);
        }

        let signal_clean = math::remove_dc_offset(signal);
        let spectrum = self.fft(&signal_clean)?;
        let magnitudes = self.magnitude_spectrum(&spectrum)?;
        let bin_width = T::from(self.sample_rate).unwrap() / T::from(self.fft_size).unwrap();
        let threshold = math::calculate_threshold(&magnitudes);

        Ok(magnitudes
            .iter()
            .enumerate()
            .filter(|&(i, &v)| {
                v > threshold
                    && (i == 0 || v > magnitudes[i - 1])
                    && (i == magnitudes.len() - 1 || v > magnitudes[i + 1])
            })
            .map(|(i, _)| T::from(i).unwrap() * bin_width)
            .collect())
    }

    pub fn chromagram(&self, signal: &[T]) -> Result<[T; 12], SignalError> {
        if signal.is_empty() {
            return Err(SignalError::EmptySignal);
        }

        let clean = math::remove_dc_offset(signal);
        let windowed = math::apply_hann_window(&clean);
        let spectrum = self.fft(&windowed)?;
        let magnitudes = self.magnitude_spectrum(&spectrum)?;

        let bin_width = T::from(self.sample_rate).unwrap() / T::from(self.fft_size).unwrap();

        let mut chroma = [T::zero(); 12];

        for (i, &mag) in magnitudes.iter().enumerate() {
            let freq = T::from(i).unwrap() * bin_width;
            if freq < T::from(20.0).unwrap() {
                continue;
            }

            let midi = T::from(69.0).unwrap()
                + T::from(12.0).unwrap() * (freq / T::from(440.0).unwrap()).log2();

            let chroma_idx = midi.round().to_i64().unwrap().rem_euclid(12) as usize;
            chroma[chroma_idx] = chroma[chroma_idx] + mag;
        }

        Ok(chroma)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    const SAMPLE_RATE: usize = 1000;
    const FFT_SIZE: usize = 200;

    fn generate_sin_wave(freq: f32, sample_rate: usize, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| ((2.0 * PI * freq * i as f32) / sample_rate as f32).sin())
            .collect()
    }

    fn generate_multi_sin_wave(freqs: &[f32]) -> Vec<f32> {
        freqs
            .iter()
            .map(|f| generate_sin_wave(*f, SAMPLE_RATE, FFT_SIZE))
            .fold(vec![0.0; FFT_SIZE], |acc, w| {
                acc.iter().zip(w.iter()).map(|(a, x)| a + x).collect()
            })
    }

    fn approx(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() < eps
    }

    fn get_signal_processor() -> Result<SignalProcessor<f32>, SignalError> {
        SignalProcessor::new(SAMPLE_RATE, FFT_SIZE)
    }

    // ============================================================
    // SIGNAL PROCESSOR CREATION TESTS
    // ============================================================

    #[test]
    fn test_signal_processor_stores_params() {
        let p = SignalProcessor::<f32>::new(48_000, 1024).expect("Failed to create processor");
        assert_eq!(p.sample_rate, 48_000);
        assert_eq!(p.fft_size, 1024);
    }

    #[test]
    fn test_signal_processor_constructor_returns_error_when_sampling_rate_non_positive() {
        let result: Result<SignalProcessor<f32>, SignalError> = SignalProcessor::new(0, FFT_SIZE);
        assert!(matches!(result, Err(SignalError::InvalidSampleRate)));
    }

    #[test]
    fn test_signal_processor_constructor_returns_error_when_fft_size_non_positive() {
        let result: Result<SignalProcessor<f32>, SignalError> =
            SignalProcessor::new(SAMPLE_RATE, 0);
        assert!(matches!(result, Err(SignalError::InvalidNumberOfSamples)));
    }

    // ============================================================
    // FFT FUNCTION TESTS
    // ============================================================

    #[test]
    fn test_fft_returns_error_when_signal_len_not_equal_to_fft_size() {
        let p = get_signal_processor().expect("Failed to create processor");
        let mut signal = vec![0.0; FFT_SIZE];
        signal.push(0.0);
        let result = p.fft(&signal);
        assert!(matches!(result, Err(SignalError::InvalidNumberOfSamples)));
    }

    #[test]
    fn test_fft_output_length_equals_to_fft_size() {
        let p = get_signal_processor().expect("Failed to create processor");
        let signal = vec![0.0; FFT_SIZE];
        let spectrum = p.fft(&signal).expect("FFT failed");
        assert_eq!(spectrum.len(), FFT_SIZE);
    }

    #[test]
    fn test_fft_constant_signal_concentrates_in_bin_zero() {
        let p = get_signal_processor().expect("Failed to create processor");
        let signal = vec![1.0; FFT_SIZE];
        let spectrum = p.fft(&signal).expect("FFT failed");

        assert!(approx(spectrum[0].re, FFT_SIZE as f32, 1e-4));
        assert!(approx(spectrum[0].im, 0.0, 1e-4));

        for i in &spectrum[1..] {
            assert!(approx(i.norm(), 0.0, 1e-4));
        }
    }

    #[test]
    fn test_fft_zero_wave_returns_all_zero() {
        let p = get_signal_processor().expect("Failed to create processor");
        let signal = vec![0.0; FFT_SIZE];
        let spectrum = p.fft(&signal).expect("FFT failed");
        assert!(spectrum.iter().all(|c| approx(c.norm(), 0.0, 1e-9)));
    }

    #[test]
    fn test_fft_sine_magnitude_peaks_at_expected_bin() {
        let p = get_signal_processor().expect("Failed to create processor");
        let k0 = 4usize;
        let freq = k0 as f32 * SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let signal = generate_sin_wave(freq, SAMPLE_RATE, FFT_SIZE);

        let spectrum = p.fft(&signal).expect("FFT failed");
        let expected = FFT_SIZE as f32 / 2.0;
        let mirror = FFT_SIZE - k0;

        assert!((spectrum[k0].norm() - expected).abs() < 1e-4);
        assert!((spectrum[mirror].norm() - expected).abs() < 1e-4);

        for (i, c) in spectrum.iter().enumerate() {
            if i != k0 && i != mirror {
                assert!(c.norm() < 1e-4);
            }
        }
    }

    #[test]
    fn test_fft_sum_of_two_sines() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_width = SAMPLE_RATE as f32 / FFT_SIZE as f32;

        let k1 = FFT_SIZE / 8;
        let k2 = FFT_SIZE / 4;
        assert!(k1 != k2 && k1 > 0 && k2 > 0 && k1 < FFT_SIZE / 2 && k2 < FFT_SIZE / 2);

        let (a1, a2) = (1.0, 0.5);
        let f1 = k1 as f32 * bin_width;
        let f2 = k2 as f32 * bin_width;

        let s1 = generate_sin_wave(f1, SAMPLE_RATE, FFT_SIZE);
        let s2 = generate_sin_wave(f2, SAMPLE_RATE, FFT_SIZE);
        let signal: Vec<f32> = s1.iter().zip(&s2).map(|(x, y)| a1 * x + a2 * y).collect();

        let spectrum = p.fft(&signal).expect("FFT failed");
        let n = FFT_SIZE as f32;

        assert!((spectrum[k1].norm() - a1 * n / 2.0).abs() < 1e-3);
        assert!((spectrum[k2].norm() - a2 * n / 2.0).abs() < 1e-3);
        assert!((spectrum[FFT_SIZE - k1].norm() - a1 * n / 2.0).abs() < 1e-3);
        assert!((spectrum[FFT_SIZE - k2].norm() - a2 * n / 2.0).abs() < 1e-3);

        let peaks = [k1, k2, FFT_SIZE - k1, FFT_SIZE - k2];
        for (i, c) in spectrum.iter().enumerate() {
            if !peaks.contains(&i) {
                assert!(c.norm() < 1e-3);
            }
        }
    }

    #[test]
    fn test_fft_energy_conservation() {
        let p = get_signal_processor().expect("Failed to create processor");
        let signal = generate_sin_wave(150.0, SAMPLE_RATE, FFT_SIZE);

        let time_energy: f32 = signal.iter().map(|x| x.powi(2)).sum();

        let spectrum = p.fft(&signal).expect("FFT failed");
        let freq_energy: f32 = spectrum.iter().map(|c| c.norm_sqr()).sum::<f32>() / FFT_SIZE as f32;

        assert!(approx(time_energy, freq_energy, 0.01));
    }

    // ============================================================
    // MAGNITUDE SPECTRUM TESTS
    // ============================================================

    #[test]
    fn test_magnitude_spectrum_should_return_half_length_of_fft_size_even_case() {
        let sp = get_signal_processor().expect("Failed to create processor");
        let signal = generate_sin_wave(32.0, SAMPLE_RATE, FFT_SIZE);
        let fft = sp.fft(&signal).expect("FFT failed");
        let mags = sp
            .magnitude_spectrum(&fft)
            .expect("Magnitude spectrum failed");
        assert_eq!(mags.len(), FFT_SIZE / 2);
    }

    #[test]
    fn test_magnitude_spectrum_should_return_half_length_of_fft_size_odd_case() {
        let sp =
            SignalProcessor::new(SAMPLE_RATE, FFT_SIZE + 1).expect("Failed to create processor");
        let signal = generate_sin_wave(32.0, SAMPLE_RATE, FFT_SIZE + 1);
        let fft = sp.fft(&signal).expect("FFT failed");
        let mags = sp
            .magnitude_spectrum(&fft)
            .expect("Magnitude spectrum failed");
        assert_eq!(mags.len(), (FFT_SIZE + 1) / 2);
    }

    #[test]
    fn test_magnitude_spectrum_returns_true_results() {
        let p = get_signal_processor().expect("Failed to create processor");
        let fft = vec![
            Complex::new(3.0, 4.0),
            Complex::new(7.0, 24.0),
            Complex::new(5.0, 12.0),
            Complex::new(6.0, 8.0),
        ];
        let expected = vec![5.0, 25.0];
        let mags = p
            .magnitude_spectrum(&fft)
            .expect("Magnitude spectrum failed");

        for (i, _) in mags.iter().enumerate() {
            assert_eq!(mags[i], expected[i]);
        }
    }

    #[test]
    fn test_magnitude_spectrum_returns_error_on_empty_spectrum() {
        let p = get_signal_processor().expect("Failed to create processor");
        let fft = vec![];
        let result = p.magnitude_spectrum(&fft);
        assert!(matches!(result, Err(SignalError::EmptySpectrum)));
    }

    // ============================================================
    // PEAK FREQUENCIES TESTS
    // ============================================================

    #[test]
    fn test_peak_frequencies_returns_freq_of_one_sign_wave() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let f = 125.0;
        let wave = generate_sin_wave(f, SAMPLE_RATE, FFT_SIZE);
        let peaks = p.peak_frequencies(&wave).expect("Peak detection failed");

        assert_eq!(peaks.len(), 1);
        assert!(approx(peaks[0], f, bin_resolution));
    }

    #[test]
    fn test_peak_frequencies_returns_freq_of_multiple_sign_wave() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let freqs = vec![126.0, 250.0, 380.0, 450.0];
        let wave = generate_multi_sin_wave(&freqs);
        let peaks = p.peak_frequencies(&wave).expect("Peak detection failed");

        assert_eq!(peaks.len(), freqs.len());
        for (idx, _) in freqs.iter().enumerate() {
            assert!(approx(freqs[idx], peaks[idx], bin_resolution));
        }
    }

    #[test]
    fn test_peak_frequencies_with_noise() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let freq = 200.0;
        let mut signal = generate_sin_wave(freq, SAMPLE_RATE, FFT_SIZE);

        let noise: Vec<f32> = (0..FFT_SIZE)
            .map(|_| (rand::random::<f32>() - 0.5) * 0.3)
            .collect();

        signal = signal.iter().zip(&noise).map(|(s, n)| s + n).collect();

        let peaks = p.peak_frequencies(&signal).expect("Peak detection failed");
        assert_eq!(peaks.len(), 1);
        assert!(approx(peaks[0], freq, bin_resolution * 2.0));
    }

    #[test]
    fn test_peak_frequencies_with_close_frequencies_distinguishable() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let freqs = vec![240.0, 250.0];
        let wave = generate_multi_sin_wave(&freqs);
        let peaks = p.peak_frequencies(&wave).expect("Peak detection failed");

        assert_eq!(peaks.len(), 2);
        assert!(approx(peaks[0], freqs[0], bin_resolution * 1.5));
        assert!(approx(peaks[1], freqs[1], bin_resolution * 1.5));
    }

    #[test]
    fn test_peak_frequencies_with_close_frequencies_indistinguishable() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let freqs = vec![245.0, 247.0];
        let wave = generate_multi_sin_wave(&freqs);
        let peaks = p.peak_frequencies(&wave).expect("Peak detection failed");

        if peaks.len() == 1 {
            let expected_avg = (freqs[0] + freqs[1]) / 2.0;
            assert!(approx(peaks[0], expected_avg, bin_resolution * 2.0));
        } else if peaks.len() == 2 {
            assert!((peaks[0] - peaks[1]).abs() <= bin_resolution * 1.5);
        } else {
            panic!("Unexpected number of peaks: {}", peaks.len());
        }
    }

    #[test]
    fn test_peak_frequencies_detect_different_amplitudes() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;

        let f1 = 150.0;
        let f2 = 350.0;
        let s1 = generate_sin_wave(f1, SAMPLE_RATE, FFT_SIZE);
        let s2: Vec<f32> = generate_sin_wave(f2, SAMPLE_RATE, FFT_SIZE)
            .iter()
            .map(|x| x * 0.1)
            .collect();

        let signal: Vec<f32> = s1.iter().zip(&s2).map(|(a, b)| a + b).collect();
        let peaks = p.peak_frequencies(&signal).expect("Peak detection failed");

        assert_eq!(peaks.len(), 1);
        assert!(approx(peaks[0], f1, bin_resolution * 2.0));
        assert!(!peaks.iter().any(|&p| approx(p, f2, bin_resolution * 2.0)));
    }

    #[test]
    fn test_peak_frequencies_invariant_to_dc_offset() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;
        let freqs = vec![120.0, 280.0, 420.0];
        let dc_offset = 15.0;

        let wave_clean = generate_multi_sin_wave(&freqs);
        let peaks_clean = p
            .peak_frequencies(&wave_clean)
            .expect("Peak detection failed");

        let wave_offset: Vec<f32> = wave_clean.iter().map(|x| x + dc_offset).collect();
        let peaks_offset = p
            .peak_frequencies(&wave_offset)
            .expect("Peak detection failed");

        assert_eq!(peaks_clean.len(), peaks_offset.len());

        for (idx, freq) in freqs.iter().enumerate() {
            assert!(approx(peaks_clean[idx], peaks_offset[idx], bin_resolution));
            assert!(approx(peaks_offset[idx], *freq, bin_resolution * 2.0));
        }
    }

    #[test]
    fn test_peak_frequencies_returns_error_on_empty_signal() {
        let p = get_signal_processor().expect("Failed to create processor");
        let signal = vec![];
        let result = p.peak_frequencies(&signal);
        assert!(matches!(result, Err(SignalError::EmptySignal)));
    }

    #[test]
    fn test_normalization_preserves_frequencies() {
        let p = get_signal_processor().expect("Failed to create processor");
        let bin_resolution = SAMPLE_RATE as f32 / FFT_SIZE as f32;

        let freq = 200.0;
        let signal = generate_sin_wave(freq, SAMPLE_RATE, FFT_SIZE);
        let peaks_clean = p.peak_frequencies(&signal).expect("Peak detection failed");

        let signal_with_dc: Vec<f32> = signal.iter().map(|x| x + 10.0).collect();
        let peaks_normalized = p
            .peak_frequencies(&signal_with_dc)
            .expect("Peak detection failed");

        assert_eq!(peaks_clean.len(), peaks_normalized.len());
        assert!(approx(peaks_clean[0], peaks_normalized[0], bin_resolution));
    }
}
