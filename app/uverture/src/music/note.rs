use std::fmt::Display;

use strum::EnumIter;

use crate::music::interval::Interval;

#[derive(Debug, Clone, Copy)]
pub struct Note {
    letter: Letter,
    accidental: Accidental,
    octave: i8,
}

impl Note {
    pub fn new(letter: Letter, accidental: Accidental, octave: i8) -> Note {
        Note {
            letter,
            accidental,
            octave,
        }
    }

    pub fn semitone(&self) -> i32 {
        (self.octave as i32 + 1) * 12 + self.letter.semitone() + self.accidental.offset()
    }

    pub fn is_enharmonic(&self, other: &Self) -> bool {
        self.semitone() == other.semitone()
    }

    pub fn interval(&self, iv: Interval) -> Note {
        let target = self.semitone() + iv.semitones as i32;
        let base_letter = self.letter.next(iv.letter_steps);

        [0, 1, -1, 2, -2, 3, -3]
            .into_iter()
            .find_map(|step| {
                let letter = base_letter.shift(step);
                let diff = target - letter.semitone();
                let k = (diff + 6).div_euclid(12);
                let offset = diff - k * 12;
                (-2..=2)
                    .contains(&offset)
                    .then(|| Note::new(letter, Accidental::from_offset(offset), (k - 1) as i8))
            })
            .expect("every pitch is within a double-flat/double-sharp of some letter")
    }

    pub fn frequency(&self) -> f64 {
        let freq_a4 = 440.0;
        let a4 = Note::new(Letter::A, Accidental::Natural, 4);

        let a4_semitone = a4.semitone();
        let self_semitone = self.semitone();

        let diff = (self_semitone - a4_semitone) as f64;

        freq_a4 * f64::powf(2.0, diff / 12.0)
    }

    pub fn pitch_class(&self) -> u8 {
        self.semitone().rem_euclid(12) as u8
    }
}
impl Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let letter = match self.letter {
            Letter::A => "A",
            Letter::B => "B",
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
            Letter::F => "F",
            Letter::G => "G",
        };
        let accidental = match self.accidental {
            Accidental::DoubleFlat => "bb",
            Accidental::Flat => "b",
            Accidental::Natural => "",
            Accidental::Sharp => "#",
            Accidental::DoubleSharp => "##",
        };
        write!(f, "{}{}", letter, accidental)
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Accidental {
    DoubleFlat,
    Flat,
    Natural,
    Sharp,
    DoubleSharp,
}

impl Accidental {
    fn offset(&self) -> i32 {
        match self {
            Accidental::DoubleFlat => -2,
            Accidental::Flat => -1,
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::DoubleSharp => 2,
        }
    }

    fn from_offset(o: i32) -> Accidental {
        match o {
            -2 => Accidental::DoubleFlat,
            -1 => Accidental::Flat,
            0 => Accidental::Natural,
            1 => Accidental::Sharp,
            2 => Accidental::DoubleSharp,
            _ => panic!("Invalid Offset"),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Letter {
    fn semitone(&self) -> i32 {
        match self {
            Letter::C => 0,
            Letter::D => 2,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 7,
            Letter::A => 9,
            Letter::B => 11,
        }
    }

    fn index(&self) -> u8 {
        match self {
            Letter::A => 0,
            Letter::B => 1,
            Letter::C => 2,
            Letter::D => 3,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 6,
        }
    }

    fn next(&self, n: u8) -> Letter {
        match (self.index() + n) % 7 {
            0 => Letter::A,
            1 => Letter::B,
            2 => Letter::C,
            3 => Letter::D,
            4 => Letter::E,
            5 => Letter::F,
            _ => Letter::G,
        }
    }

    fn shift(&self, n: i8) -> Letter {
        let idx = (self.index() as i8 + n).rem_euclid(7) as u8;
        match idx {
            0 => Letter::A,
            1 => Letter::B,
            2 => Letter::C,
            3 => Letter::D,
            4 => Letter::E,
            5 => Letter::F,
            _ => Letter::G,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::music::note::Note;

    #[test]
    fn test_semitone_a4_is_69() {
        let a4 = Note::new(super::Letter::A, super::Accidental::Natural, 4);
        assert_eq!(a4.semitone(), 69)
    }
    #[test]
    fn test_semitone_ab4_is_68() {
        let ab4 = Note::new(super::Letter::A, super::Accidental::Flat, 4);
        assert_eq!(ab4.semitone(), 68)
    }

    #[test]
    fn test_a4_enharmonic_bb4() {
        let bb4 = Note::new(super::Letter::B, super::Accidental::DoubleFlat, 4);
        let a4 = Note::new(super::Letter::A, super::Accidental::Natural, 4);

        assert!(a4.is_enharmonic(&bb4));
    }

    #[test]
    fn test_bs5_enharmonic_c6() {
        let bs5 = Note::new(super::Letter::B, super::Accidental::Sharp, 5);
        let c6 = Note::new(super::Letter::C, super::Accidental::Natural, 6);

        assert!(bs5.is_enharmonic(&c6));
    }
}

#[cfg(test)]
mod frequency_tests {
    use super::{Accidental, Letter, Note};

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn test_a4_is_440() {
        let a4 = Note::new(Letter::A, Accidental::Natural, 4);
        assert!(approx(a4.frequency(), 440.0));
    }

    #[test]
    fn test_octave_doubles_and_halves() {
        let a5 = Note::new(Letter::A, Accidental::Natural, 5);
        let a3 = Note::new(Letter::A, Accidental::Natural, 3);
        assert!(approx(a5.frequency(), 880.0));
        assert!(approx(a3.frequency(), 220.0));
    }

    #[test]
    fn test_middle_c() {
        let c4 = Note::new(Letter::C, Accidental::Natural, 4);
        assert!((c4.frequency() - 261.6256).abs() < 1e-3); // ~261.63 Hz
    }

    #[test]
    fn test_enharmonics_share_frequency() {
        let as4 = Note::new(Letter::A, Accidental::Sharp, 4);
        let bb4 = Note::new(Letter::B, Accidental::Flat, 4);
        assert!(approx(as4.frequency(), bb4.frequency()));
    }
}
