pub mod wave;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Bar, BarChart, BarGroup, Block, BorderType, Padding, Paragraph},
};

use crate::{app::App, ui::wave::Wave};

/// One place for the whole app's palette (Tokyo Night-ish).
/// Change the vibe by changing these six lines.
mod theme {
    use ratatui::style::Color;
    pub const BORDER: Color = Color::Rgb(0x3b, 0x42, 0x61); // muted slate
    pub const DIM: Color = Color::Rgb(0x56, 0x5f, 0x89); // labels, hints
    pub const WAVE: Color = Color::Rgb(0x7d, 0xcf, 0xff); // ice blue
    pub const CHORD: Color = Color::Rgb(0xbb, 0x9a, 0xf7); // soft purple
    pub const ACCENT: Color = Color::Rgb(0x7a, 0xa2, 0xf7); // chroma highlight
    pub const LIVE: Color = Color::Rgb(0x9e, 0xce, 0x6a); // green
}

const NOTE_LABELS: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

pub fn ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(9)]).split(frame.area());

    // ── waveform block ────────────────────────────────
    let channel_label = if app.channels == 1 { "mono" } else { "stereo" };
    let status = Line::from(vec![
        Span::styled("● ", Style::default().fg(theme::LIVE)),
        Span::styled(
            format!(
                "live · {} kHz · {} ",
                app.sample_rate as f32 / 1000.0,
                channel_label
            ),
            Style::default().fg(theme::DIM),
        ),
    ])
    .right_aligned();

    let wave_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER))
        .title(
            Line::from(vec![Span::styled(
                " kvothe ",
                Style::default().fg(theme::CHORD).bold(),
            )])
            .left_aligned(),
        )
        .title(status);
    let wave_inner = wave_block.inner(chunks[0]);
    frame.render_widget(wave_block, chunks[0]);
    let signal = app.waveform_history.make_contiguous();
    frame.render_widget(Wave::new(signal), wave_inner);

    // ── chord block ───────────────────────────────────
    let chord_block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme::BORDER))
        .padding(Padding::new(2, 2, 1, 0)) // left, right, top, bottom
        .title_bottom(
            Line::from(vec![
                Span::styled(" q ", Style::default().fg(theme::ACCENT)),
                Span::styled("quit ", Style::default().fg(theme::DIM)),
            ])
            .left_aligned(),
        )
        .title_bottom(
            Line::from(concat!(" kvothe v", env!("CARGO_PKG_VERSION"), " "))
                .right_aligned()
                .style(Style::default().fg(theme::DIM)),
        );
    let chord_inner = chord_block.inner(chunks[1]);
    frame.render_widget(chord_block, chunks[1]);

    let chord_chunks = Layout::horizontal([
        Constraint::Length(16),
        Constraint::Fill(1),
        Constraint::Length(14),
    ])
    .spacing(2)
    .split(chord_inner);

    // ── chord readout ─────────────────────────────────
    let (name_line, notes) = match app.chord.as_ref() {
        Some(c) => (
            Line::from(Span::styled(
                c.to_string(),
                Style::default()
                    .fg(theme::CHORD)
                    .add_modifier(Modifier::BOLD),
            )),
            c.notes()
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join(" · "),
        ),
        None => (
            Line::from(Span::styled(
                "listening…",
                Style::default()
                    .fg(theme::DIM)
                    .add_modifier(Modifier::ITALIC),
            )),
            String::new(),
        ),
    };
    let chord_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("CHORD", Style::default().fg(theme::DIM))),
        name_line,
        Line::from(Span::styled(notes, Style::default().fg(theme::DIM))),
    ]);
    frame.render_widget(chord_paragraph, chord_chunks[0]);

    // ── chroma bars ───────────────────────────────────
    let chord_tones: Vec<usize> = app
        .chord
        .as_ref()
        .map(|c| c.pitch_classes())
        .unwrap_or_default();
    let root = app.chord.as_ref().map(|c| c.root.pitch_class());

    let bars: Vec<Bar> = app
        .chroma
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let (bar_style, label_style) = if root == Some(i as u8) {
                (
                    Style::default().fg(theme::CHORD),
                    Style::default().fg(theme::CHORD).bold(),
                )
            } else if chord_tones.contains(&i) {
                (
                    Style::default().fg(theme::ACCENT),
                    Style::default().fg(theme::ACCENT),
                )
            } else {
                (
                    Style::default().fg(theme::BORDER),
                    Style::default().fg(theme::DIM),
                )
            };
            Bar::default()
                .value((v * 100.0) as u64)
                .text_value(String::new())
                .label(Line::from(NOTE_LABELS[i]).style(label_style))
                .style(bar_style)
        })
        .collect();

    let chroma_chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(2)
        .bar_gap(1)
        .max(100);
    frame.render_widget(chroma_chart, chord_chunks[1]);

    // ── level meter ───────────────────────────────────
    let db = 20.0 * app.level.max(1e-6).log10();
    let norm = ((db + 40.0) / 40.0).clamp(0.0, 1.0);

    let total = 10usize;
    let filled = (norm * total as f32).round() as usize;
    let cells: Vec<Span> = (0..total)
        .map(|i| {
            let ch = if i < filled { "▮" } else { "▯" };
            let color = if i < filled {
                match i {
                    0..=5 => theme::LIVE,
                    6..=7 => Color::Rgb(0xe0, 0xaf, 0x68), // amber
                    _ => Color::Rgb(0xf7, 0x76, 0x8e),     // red
                }
            } else {
                theme::BORDER
            };
            Span::styled(ch, Style::default().fg(color))
        })
        .collect();

    let level_paragraph = Paragraph::new(vec![
        Line::from(Span::styled("LEVEL", Style::default().fg(theme::DIM))),
        Line::from(cells),
        Line::from(Span::styled(
            format!("{:>5.1} dB", db.max(-60.0)),
            Style::default().fg(theme::DIM),
        )),
    ])
    .alignment(Alignment::Left);
    frame.render_widget(level_paragraph, chord_chunks[2]);
}
