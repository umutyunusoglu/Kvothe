use std::collections::VecDeque;

use felurian::mic::CaptureInfo;
use uverture::music::chords::Chord;

use crate::messages::AnalysisUpdateMessage;

pub struct App {
    pub sample_rate: usize,
    pub channels: usize,
    pub waveform_history: VecDeque<f64>,
    pub history_capacity: usize,
    pub chord: Option<Chord>,
    pub chroma: [f32; 12],
    pub level: f32,
}

impl App {
    pub fn new(info: CaptureInfo) -> App {
        App {
            sample_rate: info.sample_rate,
            channels: info.channels,
            waveform_history: VecDeque::with_capacity(info.sample_rate),
            history_capacity: info.sample_rate,
            chord: None,
            chroma: [0.0; 12],
            level: 0.0,
        }
    }

    pub fn apply_analysis(&mut self, u: AnalysisUpdateMessage) {
        for s in u.mono_hop {
            if self.waveform_history.len() == self.history_capacity {
                self.waveform_history.pop_front();
            }

            self.waveform_history.push_back(s);

            self.chord = u.chord;
            self.chroma = u.chroma;
            self.level = u.level;
        }
    }
}
