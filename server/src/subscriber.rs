use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc::UnboundedSender;
use turbo_hearts_api::GameEvent;

#[derive(Debug)]
pub struct Subscriber {
    tx: UnboundedSender<(GameEvent, usize)>,
    counter: AtomicUsize,
    last_event_id: usize,
}

impl Subscriber {
    pub fn new(tx: UnboundedSender<(GameEvent, usize)>, last_event_id: Option<usize>) -> Self {
        Self {
            tx,
            counter: AtomicUsize::new(1),
            last_event_id: last_event_id.unwrap_or(0),
        }
    }

    pub fn send(&self, event: GameEvent) -> bool {
        if event.is_stable() {
            let event_id = self.counter.fetch_add(1, Ordering::Relaxed);
            if event_id > self.last_event_id {
                self.tx.send((event, event_id)).is_ok()
            } else {
                true
            }
        } else {
            self.tx.send((event, 0)).is_ok()
        }
    }
}
