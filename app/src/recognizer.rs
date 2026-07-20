use felurian::mic::CaptureInfo;
use uverture::{
    music::chords::{Chord, all_chords},
    signal_processor::SignalProcessor,
};

use crate::messages::AnalysisUpdateMessage;

pub const FFT_SIZE: usize = 8192;
pub const HOP_SIZE: usize = 2048;

pub const SILENCE_THRESHOLD_DB: f32 = -45.0;

/// Streaming FFT/chord analysis over interleaved samples. `push_samples` accepts
/// any-length chunks (native feeds exact hop-sized slices; the web build feeds
/// whatever block size the browser's audio callback hands it) and internally
/// accumulates until a full hop is available.
pub struct Recognizer {
    processor: SignalProcessor<f32>,
    channels: usize,
    acc: Vec<f32>,
    window: Vec<f32>,
}

impl Recognizer {
    pub fn new(info: CaptureInfo) -> anyhow::Result<Self> {
        let processor = SignalProcessor::new(info.sample_rate, FFT_SIZE)?;

        Ok(Self {
            processor,
            channels: info.channels,
            acc: Vec::with_capacity(HOP_SIZE * info.channels),
            window: vec![0.0f32; FFT_SIZE],
        })
    }

    pub fn push_samples(&mut self, samples: &[f32]) -> Vec<AnalysisUpdateMessage> {
        self.acc.extend_from_slice(samples);

        let hop_len = HOP_SIZE * self.channels;
        let mut messages = Vec::new();

        while self.acc.len() >= hop_len {
            let raw: Vec<f32> = self.acc.drain(..hop_len).collect();
            if let Some(msg) = self.process_hop(&raw) {
                messages.push(msg);
            }
        }

        messages
    }

    fn process_hop(&mut self, raw: &[f32]) -> Option<AnalysisUpdateMessage> {
        let channels = self.channels;

        let mono: Vec<f32> = raw
            .chunks_exact(channels)
            .map(|f| f.iter().sum::<f32>() / channels as f32)
            .collect();

        self.window.copy_within(HOP_SIZE.., 0);
        self.window[FFT_SIZE - HOP_SIZE..].copy_from_slice(&mono);

        let chroma = self.processor.chromagram(&self.window).ok()?;

        let mut best_match_score = -2.0;
        let mut best_match_chord: Option<Chord> = None;

        for c in all_chords() {
            let template = c.chroma_template();
            let sim = cosine_similarity(&template, &chroma);
            if sim > best_match_score {
                best_match_score = sim;
                best_match_chord = Some(c.clone());
            }
        }

        let level = mono.iter().map(|s| s * s).sum::<f32>() / mono.len() as f32;
        let mono_hop: Vec<f64> = mono.iter().map(|&s| s as f64).collect();

        let db = 20.0 * level.max(1e-6).log10();
        if db < SILENCE_THRESHOLD_DB {
            best_match_chord = None;
        }

        Some(AnalysisUpdateMessage {
            chord: best_match_chord,
            chroma,
            level,
            mono_hop,
        })
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (norm_a * norm_b)
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{sync::mpsc, thread, time::Duration};

    use felurian::mic::CaptureInfo;
    use ringbuf::{
        HeapCons,
        traits::{Consumer, Observer},
    };

    use super::{HOP_SIZE, Recognizer};
    use crate::messages::AnalysisUpdateMessage;

    pub fn spawn_recognizer(
        consumer: HeapCons<f32>,
        info: CaptureInfo,
    ) -> mpsc::Receiver<AnalysisUpdateMessage> {
        let (tx, rx) = mpsc::channel::<AnalysisUpdateMessage>();

        thread::spawn(move || {
            let Ok(recognizer) = Recognizer::new(info) else {
                eprintln!("Failed to create signal processor");

                return;
            };
            analysis_loop(consumer, info, recognizer, tx);
        });

        rx
    }

    fn analysis_loop(
        mut consumer: HeapCons<f32>,
        info: CaptureInfo,
        mut recognizer: Recognizer,
        tx: mpsc::Sender<AnalysisUpdateMessage>,
    ) {
        let channels = info.channels as usize;
        let mut raw = vec![0.0f32; HOP_SIZE * channels];

        loop {
            if consumer.occupied_len() < raw.len() {
                std::thread::sleep(Duration::from_millis(5));
                continue;
            }

            consumer.pop_slice(&mut raw);

            for msg in recognizer.push_samples(&raw) {
                if tx.send(msg).is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::spawn_recognizer;
