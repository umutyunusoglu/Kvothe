use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    symbols::Marker,
    widgets::{
        Widget,
        canvas::{Canvas, Line},
    },
};

pub struct Wave<'a> {
    signal: &'a [f64],
}

impl<'a> Widget for Wave<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let buckets: usize = area.width.max(1) as usize * 2; // Braille x-resolution
        let peaks = Self::downsample_minmax(self.signal, buckets);

        let max_amp = peaks
            .iter()
            .flat_map(|&(min, max)| [min.abs(), max.abs()])
            .fold(0.0_f64, f64::max)
            .max(0.05);

        let canvas = Canvas::default()
            .x_bounds([0.0, peaks.len() as f64])
            .y_bounds([-max_amp, max_amp * 1.2])
            .marker(Marker::Braille)
            .paint(move |ctx| {
                for (i, &(min, max)) in peaks.iter().enumerate() {
                    ctx.draw(&Line {
                        x1: i as f64,
                        y1: min,
                        x2: i as f64,
                        y2: max,
                        color: Color::Rgb(0x7d, 0xcf, 0xff),
                    });
                }
            });

        canvas.render(area, buf);
    }
}

impl<'a> Wave<'a> {
    pub fn new(signal: &'a [f64]) -> Wave<'a> {
        Wave { signal }
    }
    fn downsample_minmax(signal: &[f64], buckets: usize) -> Vec<(f64, f64)> {
        if signal.is_empty() || buckets == 0 {
            return vec![];
        }
        let chunk_size = (signal.len() as f64 / buckets as f64).ceil() as usize;
        signal
            .chunks(chunk_size.max(1))
            .map(|c| {
                let min = c.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = c.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                (min, max)
            })
            .collect()
    }
}
