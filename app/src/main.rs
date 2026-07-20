pub mod app;
pub mod messages;
pub mod recognizer;
pub mod ui;
pub mod utils;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    native::main()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    web::start();
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::sync::mpsc;
    use std::time::Duration;

    use anyhow::Result;
    use crossterm::event::{self, Event, KeyCode};
    use ratatui::DefaultTerminal;

    use crate::app::App;
    use crate::messages::AnalysisUpdateMessage;
    use crate::recognizer::spawn_recognizer;
    use crate::ui::ui;

    pub fn main() -> Result<()> {
        let (_mic_handle, audio_consumer, capture_info) = felurian::mic::spawn_capture()?;
        let analysix_rx = spawn_recognizer(audio_consumer, capture_info);

        let terminal = ratatui::init();
        let mut app = App::new(capture_info);
        let result = run_app(terminal, &mut app, analysix_rx);
        ratatui::restore();

        result
    }

    fn run_app(
        mut terminal: DefaultTerminal,
        app: &mut App,
        analysis_rx: mpsc::Receiver<AnalysisUpdateMessage>,
    ) -> Result<()> {
        loop {
            while let Ok(update) = analysis_rx.try_recv() {
                app.apply_analysis(update);
            }

            terminal.draw(|f| ui(f, app))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == event::KeyEventKind::Release {
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
