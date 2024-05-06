use std::sync::Mutex;

use crossbeam::channel::{Receiver, Sender};
use wezterm_term::{KeyCode, KeyModifiers, TerminalSize};

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

    pub fn paste(&self, content: String) {
        self.send_event(UserEvent::Paste(content));
    }

    pub fn key_down(&self, key: KeyCode, mods: KeyModifiers) {
        self.send_event(UserEvent::Keydown(key, mods));
    }

    pub fn resize(&self, terminal_size: (f32, f32), cell_size: (f32, f32), row_spacing: usize) {
        let (terminal_width, terminal_height) = terminal_size;
        let (cell_width, cell_height) = cell_size;

        let cols = f32::max(terminal_width / cell_width, 1.) as usize;
        let rows = f32::max(terminal_height / cell_height, 1.) as usize;

        let total_row_spacing = row_spacing * rows;
        let terminal_height_with_row_spacing = terminal_height - total_row_spacing as f32;

        let rows = f32::max(terminal_height_with_row_spacing / cell_height, 1.) as usize;

        self.send_event(UserEvent::Resize(TerminalSize {
            rows,
            cols,
            pixel_width: terminal_width as usize,
            pixel_height: terminal_height as usize,
            dpi: 1,
        }));
    }

    pub fn scroll(&self, delta_y: f64) {
        self.send_event(UserEvent::Scroll(delta_y));
    }

    pub fn send_event(&self, event: UserEvent) {
        self.terminal_channel.0.send(event).unwrap();
    }
}
