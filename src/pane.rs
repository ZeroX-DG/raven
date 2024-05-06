use std::sync::Mutex;

use crossbeam::channel::{Receiver, Sender};
use wezterm_term::TerminalSize;

use crate::terminal_loop::{create_terminal, TerminalEvent, UserEvent};

pub type PaneId = usize;
static PANE_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

pub fn alloc_pane_id() -> PaneId {
    PANE_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
}

pub struct Pane {
    pub id: PaneId,
    terminal_channel: (Sender<UserEvent>, Receiver<TerminalEvent>),
    title: Mutex<String>,
}

impl PartialEq for Pane {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Pane {
    pub fn new(id: PaneId, size: TerminalSize) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            terminal_channel: create_terminal(size)?,
            title: Mutex::new(format!("Terminal #{}", id)),
        })
    }

    pub fn title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn terminal_events(&self) -> Receiver<TerminalEvent> {
        self.terminal_channel.1.clone()
    }

    pub fn user_event_sender(&self) -> Sender<UserEvent> {
        self.terminal_channel.0.clone()
    }
}
