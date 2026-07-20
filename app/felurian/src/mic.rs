#[derive(Debug, Clone, Copy)]
pub struct CaptureInfo {
    pub sample_rate: usize,
    pub channels: usize,
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::{sync::mpsc, thread};

    use anyhow::{Context, Result, bail};
    use cpal::{
        SampleFormat,
        traits::{DeviceTrait, HostTrait, StreamTrait},
    };
    use ringbuf::{
        HeapCons, HeapProd, HeapRb,
        traits::{Producer, Split},
    };

    use super::CaptureInfo;

    pub struct MicHandle {
        _control: mpsc::Sender<()>,
    }

    const RING_CAPACITY: usize = 65_536;

    pub fn spawn_capture() -> Result<(MicHandle, HeapCons<f32>, CaptureInfo)> {
        let rb = HeapRb::<f32>::new(RING_CAPACITY);
        let (producer, consumer) = rb.split();
        let (control_tx, control_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel::<Result<CaptureInfo>>();

        thread::spawn(move || {
            let built = build_stream(producer);
            match built {
                Ok((stream, info)) => {
                    let _ = ready_tx.send(Ok(info));
                    let _stream = stream;
                    let _ = control_rx.recv();
                }
                Err(e) => {
                    let _ = ready_tx.send(Err(e));
                }
            }
        });

        let info = ready_rx
            .recv()
            .context("Capture thread died before reporting status")??;

        Ok((
            MicHandle {
                _control: control_tx,
            },
            consumer,
            info,
        ))
    }

    fn build_stream(mut producer: HeapProd<f32>) -> Result<(cpal::Stream, CaptureInfo)> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("Microphone not configured")?;
        let config = device
            .default_input_config()
            .context("Config unavaliable!")?;

        let info = CaptureInfo {
            sample_rate: config.sample_rate() as usize,
            channels: config.channels() as usize,
        };

        let sample_format = config.sample_format();
        let stream_config: cpal::StreamConfig = config.into();
        let err_fn = |err| eprintln!("Stream Error: {err}");
        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Lock-free, allocation-free. Returns how many samples fit;
                    // if the buffer is full, the newest samples are dropped.
                    let _ = producer.push_slice(data);
                },
                err_fn,
                None,
            )?,
            SampleFormat::I16 => device.build_input_stream(
                stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    for &s in data {
                        let _ = producer.try_push(s as f32 / i16::MAX as f32);
                    }
                },
                err_fn,
                None,
            )?,
            SampleFormat::U16 => device.build_input_stream(
                stream_config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    for &s in data {
                        let _ = producer.try_push((s as f32 / u16::MAX as f32) * 2.0 - 1.0);
                    }
                },
                err_fn,
                None,
            )?,
            other => bail!("Unsupported sample format: {other:?}"),
        };

        stream.play().context("Failed to start stream")?;

        Ok((stream, info))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use native::{MicHandle, spawn_capture};
