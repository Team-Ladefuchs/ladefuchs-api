use std::time::Duration;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::sleep,
};

#[derive(Debug)]
#[allow(dead_code)]
enum Cmd {
    Reset,
    SetDuration(Duration),
}

pub type Interval = Receiver<()>;

#[allow(dead_code)]
pub struct Timer {
    tx: Sender<Cmd>,
}

impl Timer {
    pub fn new(mut time: Duration) -> (Self, Receiver<()>) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (inner_tx, inner_rx) = tokio::sync::mpsc::channel(1);

        tokio::task::spawn(async move {
            if let Err(_) = inner_tx.send(()).await {
                return;
            }
            let mut timeout = sleep(time);
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
                timeout = sleep(time);
                if let Err(_) = inner_tx.send(()).await {
                    break;
                }
            }
        });

        (Self { tx }, inner_rx)
    }

    #[allow(dead_code)]
    pub async fn reset(&self) {
        self.tx
            .send(Cmd::Reset)
            .await
            .expect("Timer task disappeared");
    }

    // pub async fn set_interval(&self, interval: Duration) {
    //     self.tx
    //         .send(Cmd::SetDuration(interval))
    //         .await
    //         .expect("Timer task disappeared");
    // }
}
