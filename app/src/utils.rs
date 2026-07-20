use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

pub fn generate_sample_signal(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let wave = (t * std::f64::consts::TAU * 5.0).sin()
                + 0.4 * (t * std::f64::consts::TAU * 13.0).sin()
                + 0.15 * (t * std::f64::consts::TAU * 37.0).sin();
            wave * 0.5 // genliği -1.0..1.0 civarına çekmek için
        })
        .collect()
}

pub fn fan_out<T: Clone + Send + 'static>(source: Receiver<T>, n: usize) -> Vec<Receiver<T>> {
    let mut senders: Vec<Sender<T>> = Vec::with_capacity(n);
    let mut recievers: Vec<Receiver<T>> = Vec::with_capacity(n);

    for _ in 0..n {
        let (rx, tx) = mpsc::channel();
        senders.push(rx);
        recievers.push(tx);
    }

    thread::spawn(move || {
        for item in source.iter() {
            let last = senders.len() - 1;
            for tx in &senders[..last] {
                let _ = tx.send(item.clone());
            }
            let _ = senders[last].send(item);
        }
    });

    recievers
}
