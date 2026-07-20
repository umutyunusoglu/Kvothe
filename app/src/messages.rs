use uverture::music::chords::Chord;

pub struct AnalysisUpdateMessage {
    pub chord: Option<Chord>,
    pub chroma: [f32; 12],
    pub level: f32,
    pub mono_hop: Vec<f64>,
}
