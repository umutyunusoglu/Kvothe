use std::{
    fmt::{self, Display},
    sync::OnceLock,
};

use strum::{EnumIter, IntoEnumIterator};

use crate::music::{
    interval::Interval,
    note::{Accidental, Letter, Note},
};
#[derive(Debug, Clone, Copy)]
pub struct Chord {
    pub root: Note,
    pub chord_type: ChordType,
}

impl Chord {
    pub fn new(root: Note, chord_type: ChordType) -> Chord {
        Chord { root, chord_type }
    }

    pub fn notes(&self) -> Vec<Note> {
        self.chord_type
            .intervals()
            .iter()
            .map(|&iv| self.root.interval(iv))
            .collect()
    }

    pub fn frequencies(&self) -> Vec<f64> {
        self.notes().iter().map(|n| n.frequency()).collect()
    }

    pub fn pitch_classes(&self) -> Vec<usize> {
        self.notes()
            .iter()
            .map(|n| n.pitch_class() as usize)
            .collect()
    }

    pub fn chroma_template(&self) -> [f32; 12] {
        let mut template = [0.0; 12];

        for pc in self.pitch_classes() {
            template[pc as usize] = 1.0;
        }

        template
    }
}
impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.root, self.chord_type)
    }
}
#[derive(Debug, Clone, Copy, EnumIter)]
pub enum ChordType {
    Major,
    Maj7,
    Dom7,
    Minor,
    Min7,
    MinMaj7,
    Dim,
    Dim7,
    HalfDim7,
}

impl ChordType {
    fn intervals(&self) -> &'static [Interval] {
        use Interval as I;
        match self {
            ChordType::Major => &[I::UNISON, I::MAJOR_THIRD, I::PERFECT_FIFTH],
            ChordType::Minor => &[I::UNISON, I::MINOR_THIRD, I::PERFECT_FIFTH],
            ChordType::Dim => &[I::UNISON, I::MINOR_THIRD, I::DIMINISHED_FIFTH],

            ChordType::Maj7 => &[
                I::UNISON,
                I::MAJOR_THIRD,
                I::PERFECT_FIFTH,
                I::MAJOR_SEVENTH,
            ],
            ChordType::Dom7 => &[
                I::UNISON,
                I::MAJOR_THIRD,
                I::PERFECT_FIFTH,
                I::MINOR_SEVENTH,
            ],
            ChordType::Min7 => &[
                I::UNISON,
                I::MINOR_THIRD,
                I::PERFECT_FIFTH,
                I::MINOR_SEVENTH,
            ],
            ChordType::MinMaj7 => &[
                I::UNISON,
                I::MINOR_THIRD,
                I::PERFECT_FIFTH,
                I::MAJOR_SEVENTH,
            ],
            ChordType::HalfDim7 => &[
                I::UNISON,
                I::MINOR_THIRD,
                I::DIMINISHED_FIFTH,
                I::MINOR_SEVENTH,
            ],
            ChordType::Dim7 => &[
                I::UNISON,
                I::MINOR_THIRD,
                I::DIMINISHED_FIFTH,
                I::DIMINISHED_SEVENTH,
            ],
        }
    }
}

impl fmt::Display for ChordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ChordType::Major => "",
            ChordType::Maj7 => "maj7",
            ChordType::Dom7 => "7",
            ChordType::Minor => "m",
            ChordType::Min7 => "m7",
            ChordType::MinMaj7 => "m(maj7)",
            ChordType::Dim => "dim",
            ChordType::Dim7 => "dim7",
            ChordType::HalfDim7 => "m7b5",
        };
        write!(f, "{}", text)
    }
}

pub fn all_chords() -> &'static Vec<Chord> {
    static CHORDS: OnceLock<Vec<Chord>> = OnceLock::new();
    CHORDS.get_or_init(|| {
        // One canonical spelling per pitch class (matches the UI's chroma
        // labels) so candidates never tie on an enharmonic duplicate, e.g.
        // B# vs C or C## vs D scoring identically and winning on iteration
        // order alone.
        let roots = [
            (Letter::C, Accidental::Natural),
            (Letter::C, Accidental::Sharp),
            (Letter::D, Accidental::Natural),
            (Letter::D, Accidental::Sharp),
            (Letter::E, Accidental::Natural),
            (Letter::F, Accidental::Natural),
            (Letter::F, Accidental::Sharp),
            (Letter::G, Accidental::Natural),
            (Letter::G, Accidental::Sharp),
            (Letter::A, Accidental::Natural),
            (Letter::A, Accidental::Sharp),
            (Letter::B, Accidental::Natural),
        ];

        let mut v: Vec<Chord> = Vec::new();
        for (letter, accidental) in roots {
            let note = Note::new(letter, accidental, 4);

            for chord_type in ChordType::iter() {
                v.push(Chord::new(note, chord_type));
            }
        }
        v
    })
}
#[cfg(test)]
mod tests {
    use super::{Chord, ChordType};
    use crate::music::note::{Accidental, Letter, Note};

    fn chord(letter: Letter, acc: Accidental, octave: i8, ct: ChordType) -> Chord {
        Chord {
            root: Note::new(letter, acc, octave),
            chord_type: ct,
        }
    }

    fn semitones(c: &Chord) -> Vec<i32> {
        c.notes().iter().map(|n| n.semitone()).collect()
    }

    // --- notes() ---

    #[test]
    fn test_c_major_notes() {
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Major);
        assert_eq!(semitones(&c), vec![60, 64, 67]); // C4 E4 G4
    }

    #[test]
    fn test_c_maj7_notes() {
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Maj7);
        assert_eq!(semitones(&c), vec![60, 64, 67, 71]); // C4 E4 G4 B4
    }

    #[test]
    fn test_a_minor_crosses_octave() {
        // üçlü ve beşli B->C sınırını geçip üst oktava düşer
        let c = chord(Letter::A, Accidental::Natural, 4, ChordType::Minor);
        assert_eq!(semitones(&c), vec![69, 72, 76]); // A4 C5 E5
    }

    #[test]
    fn test_c_dim7_double_flat() {
        // yedili Bbb (çift bemol)
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Dim7);
        assert_eq!(semitones(&c), vec![60, 63, 66, 69]); // C Eb Gb Bbb
    }

    #[test]
    fn test_d_sharp_major_double_sharp() {
        // üçlü F## (çift diyez)
        let c = chord(Letter::D, Accidental::Sharp, 4, ChordType::Major);
        assert_eq!(semitones(&c), vec![63, 67, 70]); // D#4 F##4 A#4
    }

    // --- nota sayısı ---

    #[test]
    fn test_triads_have_three_notes() {
        for ct in [ChordType::Major, ChordType::Minor, ChordType::Dim] {
            assert_eq!(
                chord(Letter::C, Accidental::Natural, 4, ct).notes().len(),
                3
            );
        }
    }

    #[test]
    fn test_sevenths_have_four_notes() {
        for ct in [
            ChordType::Maj7,
            ChordType::Dom7,
            ChordType::Min7,
            ChordType::MinMaj7,
            ChordType::HalfDim7,
            ChordType::Dim7,
        ] {
            assert_eq!(
                chord(Letter::C, Accidental::Natural, 4, ct).notes().len(),
                4
            );
        }
    }

    // --- pitch_classes() ---

    #[test]
    fn test_c_major_pitch_classes() {
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Major);
        assert_eq!(c.pitch_classes(), vec![0, 4, 7]); // C E G
    }

    #[test]
    fn test_pitch_classes_octave_independent() {
        let low = chord(Letter::C, Accidental::Natural, 2, ChordType::Maj7);
        let high = chord(Letter::C, Accidental::Natural, 6, ChordType::Maj7);
        assert_eq!(low.pitch_classes(), high.pitch_classes());
    }

    // --- chroma_template() ---

    #[test]
    fn test_c_major_chroma_template() {
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Major);
        let mut expected = [0.0; 12];
        expected[0] = 1.0; // C
        expected[4] = 1.0; // E
        expected[7] = 1.0; // G
        assert_eq!(c.chroma_template(), expected);
    }

    #[test]
    fn test_chroma_template_sums_to_note_count() {
        let c = chord(Letter::C, Accidental::Natural, 4, ChordType::Dom7);
        let sum: f32 = c.chroma_template().iter().sum();
        assert_eq!(sum, 4.0); // 4 ayrı pitch class
    }

    // --- frequencies() ---

    #[test]
    fn test_frequencies_root_is_440() {
        let c = chord(Letter::A, Accidental::Natural, 4, ChordType::Major);
        let freqs = c.frequencies();
        assert_eq!(freqs.len(), 3);
        assert!((freqs[0] - 440.0).abs() < 1e-6); // kök A4
    }

    #[test]
    fn test_double_flat_dim7_does_not_panic() {
        let c = chord(Letter::C, Accidental::DoubleFlat, 4, ChordType::Dim7);
        assert_eq!(c.pitch_classes(), vec![10, 1, 4, 7]);
    }

    #[test]
    fn test_all_chords_do_not_panic() {
        for chord in super::all_chords() {
            chord.notes();
        }
    }
}
