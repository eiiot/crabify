use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};
use std::time::Duration;
use tokio::sync::mpsc;

use crate::action::Event;

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    _tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            if key.kind == KeyEventKind::Press {
                                let _ = tx.send(Event::Key(key));
                            }
                        }
                        Ok(CrosstermEvent::Resize(w, h)) => {
                            let _ = tx.send(Event::Resize(w, h));
                        }
                        _ => {}
                    }
                } else {
                    let _ = tx.send(Event::Tick);
                }
            }
        });

        Self { rx, _tx }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
