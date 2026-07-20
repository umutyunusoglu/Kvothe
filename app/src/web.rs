use std::cell::RefCell;

use ratzilla::{
    DomBackend, WebRenderer,
    ratatui::{Terminal, layout::Alignment, widgets::Paragraph},
};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AudioContext, AudioWorkletNode, MediaStream, MediaStreamConstraints, MessageEvent, window,
};

use felurian::mic::CaptureInfo;

use crate::app::App;
use crate::recognizer::Recognizer;
use crate::ui::ui;

const WORKLET_MODULE_URL: &str = "capture-processor.js";
const WORKLET_PROCESSOR_NAME: &str = "capture-processor";

enum WebState {
    Loading,
    Ready { app: App, recognizer: Recognizer },
    Error(String),
}

thread_local! {
    static STATE: RefCell<WebState> = RefCell::new(WebState::Loading);
}

pub fn start() {
    console_error_panic_hook::set_once();

    let backend = DomBackend::new().expect("failed to create DOM backend");
    let terminal = Terminal::new(backend).expect("failed to create terminal");

    terminal.draw_web(move |f| {
        STATE.with(|state| match &mut *state.borrow_mut() {
            WebState::Ready { app, .. } => ui(f, app),
            WebState::Loading => render_status(f, "requesting microphone access..."),
            WebState::Error(msg) => render_status(f, msg),
        });
    });

    wasm_bindgen_futures::spawn_local(async {
        if let Err(err) = request_mic().await {
            let msg = err
                .as_string()
                .unwrap_or_else(|| "failed to start microphone capture".to_string());
            STATE.with(|state| *state.borrow_mut() = WebState::Error(msg));
        }
    });
}

fn render_status(f: &mut ratzilla::ratatui::Frame, msg: &str) {
    f.render_widget(Paragraph::new(msg).alignment(Alignment::Center), f.area());
}

async fn request_mic() -> Result<(), JsValue> {
    let window = window().ok_or_else(|| JsValue::from_str("no global window"))?;
    let media_devices = window.navigator().media_devices()?;

    let constraints = MediaStreamConstraints::new();
    constraints.set_audio_bool(true);
    let stream_promise = media_devices.get_user_media_with_constraints(&constraints)?;
    let stream: MediaStream = JsFuture::from(stream_promise).await?.dyn_into()?;

    let audio_ctx = AudioContext::new()?;
    let worklet = audio_ctx.audio_worklet()?;
    JsFuture::from(worklet.add_module(WORKLET_MODULE_URL)?).await?;

    let source = audio_ctx.create_media_stream_source(&stream)?;
    let worklet_node = AudioWorkletNode::new(&audio_ctx, WORKLET_PROCESSOR_NAME)?;
    source.connect_with_audio_node(&worklet_node)?;

    let info = CaptureInfo {
        sample_rate: audio_ctx.sample_rate() as usize,
        channels: 1,
    };
    let recognizer = Recognizer::new(info).map_err(|e| JsValue::from_str(&format!("{e}")))?;
    let app = App::new(info);

    STATE.with(|state| *state.borrow_mut() = WebState::Ready { app, recognizer });

    let port = worklet_node.port()?;
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Ok(array) = event.data().dyn_into::<js_sys::Float32Array>() else {
            return;
        };
        let samples = array.to_vec();

        STATE.with(|state| {
            if let WebState::Ready { app, recognizer } = &mut *state.borrow_mut() {
                for msg in recognizer.push_samples(&samples) {
                    app.apply_analysis(msg);
                }
            }
        });
    });
    port.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    Ok(())
}
