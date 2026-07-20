use ratatui::widgets::{Paragraph, Widget};

struct ChordPane {
    current_chord: Chord,
}

impl ChordPane {
    fn new() -> Self {
        Self {
            current_chord: Chord::default(),
        }
    }
}

impl Widget for ChordPane {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let chord_text = Paragraph::new(self.current_chord.text);

        chord_text.render(area, buf);
    }
}

#[derive(Debug, Default)]
struct Chord {
    text: String,
}
