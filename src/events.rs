use std::{fmt::Debug, sync::{Arc, Mutex, OnceLock}};

use crate::core::pane::PaneId;

static EVENTS: OnceLock<Arc<Events>> = OnceLock::new();

pub struct Events {
    subscribers: Mutex<Vec<Box<dyn FnMut(Event) + Send + Sync>>>
}

#[derive(Debug, Clone)]
pub enum Event {
    PaneOutput(PaneId),
    PaneTitle {
        pane_id: PaneId,
        title: String,
    },
}

impl Debug for Events {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Events")
    }
}

impl Events {
    fn new() -> Self {
        Self {
            subscribers: Mutex::new(Vec::new())
        }
    }

    pub fn get() -> Arc<Self> {
        if let Some(events) = EVENTS.get() {
            events.clone()
        } else {
            let events = Arc::new(Events::new());
            EVENTS.set(events.clone()).expect("Unable to set events");
            events
        }
    }

    pub fn emit(&self, event: Event) {
        let mut subscribers = self.subscribers.lock().unwrap();

        for subscriber in subscribers.iter_mut() {
            subscriber(event.clone());
        }
    }

    pub fn subscribe<F: FnMut(Event) + 'static + Send + Sync>(&self, subscriber: F) {
        self.subscribers.lock().unwrap().push(Box::new(subscriber));
    }
}