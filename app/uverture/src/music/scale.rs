use crate::music::{interval::Interval, note::Note};

const MAJOR_SCALE: [Interval; 7] = {
    use Interval as I;
    [
        I::UNISON,
        I::MAJOR_SECOND,
        I::MAJOR_THIRD,
        I::PERFECT_FOURTH,
        I::PERFECT_FIFTH,
        I::MAJOR_SIXTH,
        I::MAJOR_SEVENTH,
    ]
};

pub fn major_scale(root: &Note) -> Vec<Note> {
    MAJOR_SCALE.iter().map(|&iv| root.interval(iv)).collect()
}

#[cfg(test)]
mod test {
    use crate::music::{
        note::{Accidental, Letter, Note},
        scale::major_scale,
    };

    #[test]
    fn test_c_major_scale() {
        let scale = major_scale(&Note::new(Letter::C, Accidental::Natural, 4));
        let pitches: Vec<i32> = scale.iter().map(|n| n.semitone()).collect();
        assert_eq!(pitches, vec![60, 62, 64, 65, 67, 69, 71]);
    }
}
