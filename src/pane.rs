use std::sync::Mutex;

use wezterm_term::TerminalSize;

use crate::terminal_loop::{create_terminal, TerminalBridge};

pub type PaneId = usize;
static PANE_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

pub fn alloc_pane_id() -> PaneId {
    PANE_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
}

pub struct Pane {
    pub id: PaneId,
    terminal_bridge: TerminalBridge,
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
            terminal_bridge: create_terminal(size)?,
            title: Mutex::new(format!("Terminal #{}", id)),
        })
    }

    pub fn title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn terminal_bridge(&self) -> &TerminalBridge {
        &self.terminal_bridge
    }

    pub fn close(&self) {
        // TODO: Handle exiting in the main app state
        std::process::exit(0);
    }
}
