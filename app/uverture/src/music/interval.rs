#[derive(Clone, Copy)]
pub struct Interval {
    pub(crate) letter_steps: u8,
    pub(crate) semitones: u8,
}

impl Interval {
    pub const UNISON: Interval = Interval {
        letter_steps: 0,
        semitones: 0,
    };
    pub const MAJOR_SECOND: Interval = Interval {
        letter_steps: 1,
        semitones: 2,
    };
    pub const MINOR_THIRD: Interval = Interval {
        letter_steps: 2,
        semitones: 3,
    };
    pub const MAJOR_THIRD: Interval = Interval {
        letter_steps: 2,
        semitones: 4,
    };
    pub const PERFECT_FOURTH: Interval = Interval {
        letter_steps: 3,
        semitones: 5,
    };
    pub const DIMINISHED_FIFTH: Interval = Interval {
        letter_steps: 4,
        semitones: 6,
    };
    pub const PERFECT_FIFTH: Interval = Interval {
        letter_steps: 4,
        semitones: 7,
    };
    pub const MAJOR_SIXTH: Interval = Interval {
        letter_steps: 5,
        semitones: 9,
    };
    pub const DIMINISHED_SEVENTH: Interval = Interval {
        letter_steps: 6,
        semitones: 9,
    };
    pub const MINOR_SEVENTH: Interval = Interval {
        letter_steps: 6,
        semitones: 10,
    };
    pub const MAJOR_SEVENTH: Interval = Interval {
        letter_steps: 6,
        semitones: 11,
    };
}
