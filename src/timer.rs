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
enum Command {
    Reset,
    Restart,
    SetDuration(Duration),
}

pub type Interval = Receiver<()>;

pub struct Timer {
    tx: Sender<Command>,
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
                let do_trigger = tokio::select! {
                    _ =  timeout => true,
                    Some(cmd) = rx.recv() => {
                        match cmd {
                            Command::Reset => {
                                tracing::trace!("Received timer reset");
                                true
                            },
                            Command::Restart => {
                                false
                            },
                            Command::SetDuration(duration) => {
                                time = duration;
                                true
                            }
                        }
                    }
                };

                if do_trigger {
                    if let Err(_) = inner_tx.send(()).await {
                        break;
                    }
                }

                timeout = sleep(time);

                *next_clone.lock().await = timeout.deadline();
            }
        });

        (Self { tx, next }, inner_rx)
    }

    ///
    /// Return the next expiration time for the timer
    ///
    pub async fn next(&self) -> Result<chrono::Duration, chrono::OutOfRangeError> {
        let now = Instant::now();
        chrono::Duration::from_std(*self.next.lock().await - now)
    }

    ///
    /// Restarts the time without triggering
    ///
    pub async fn restart(&self) {
        self.tx
            .send(Command::Restart)
            .await
            .expect("Timer task disappeared");
    }
}
