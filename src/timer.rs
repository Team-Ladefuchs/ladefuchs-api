use std::{sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{Receiver, Sender},
        Mutex,
    },
    time::{sleep, Instant},
};

#[derive(Debug)]
#[allow(dead_code)]
enum Cmd {
    Reset,
    SetDuration(Duration),
}

pub type Interval = Receiver<()>;

pub struct Timer {
    tx: Sender<Cmd>,
    next: Arc<Mutex<Instant>>,
}

impl Timer {
    pub fn new(mut time: Duration) -> (Self, Receiver<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (inner_tx, inner_rx) = tokio::sync::mpsc::channel(1);
        let mut timeout = sleep(time);
        let next = Arc::new(Mutex::new(timeout.deadline()));

        let next_clone = next.clone();
        tokio::task::spawn(async move {
            if let Err(_) = inner_tx.send(()).await {
                return;
            }

            loop {
                tokio::select! {
                    _ =  timeout => (),
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            Cmd::Reset => {
                                tracing::trace!("Received timer reset");
                            }
                            Cmd::SetDuration(duration) => {
                                time = duration;
                            }
                        }
                    }
                }
                if let Err(_) = inner_tx.send(()).await {
                    break;
                }
                timeout = sleep(time);

                *next_clone.lock().await = timeout.deadline();
            }
        });

        (Self { tx, next }, inner_rx)
    }

    #[allow(dead_code)]
    pub async fn reset(&self) {
        self.tx
            .send(Cmd::Reset)
            .await
            .expect("Timer task disappeared");
    }
    pub async fn next(&self) -> Result<chrono::Duration, chrono::OutOfRangeError> {
        let now = Instant::now();
        chrono::Duration::from_std(*self.next.lock().await - now)
    }

    // pub async fn set_interval(&self, interval: Duration) {
    //     self.tx
    //         .send(Cmd::SetDuration(interval))
    //         .await
    //         .expect("Timer task disappeared");
    // }
}
