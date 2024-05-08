use std::sync::Arc;

use freya::prelude::*;
use wezterm_term::{KeyCode, KeyModifiers, TerminalSize};

use crate::{pane::Pane, terminal_loop::UserEvent};

#[derive(Clone)]
pub struct UseTerminal {
    pane: Arc<Pane>,
}

impl UseTerminal {
    pub fn paste(&self, content: String) {
        self.send_event(UserEvent::Paste(content));
    }

    pub fn key_down(&self, key: KeyCode, mods: KeyModifiers) {
        self.send_event(UserEvent::Keydown(key, mods));
    }

    pub fn resize(&self, terminal_size: (f32, f32), cell_size: (f32, f32), row_spacing: u16) {
        let (terminal_width, terminal_height) = terminal_size;
        let (cell_width, cell_height) = cell_size;

        let cols = f32::max(terminal_width / cell_width, 1.) as usize;
        let rows = f32::max(terminal_height / cell_height, 1.) as usize;

        let total_row_spacing = row_spacing as usize * rows;
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
        self.pane
            .terminal_bridge()
            .user_event_sender()
            .send(event)
            .ok();
    }
}

pub fn use_terminal(pane: Arc<Pane>) -> UseTerminal {
    use_hook(|| UseTerminal { pane })
}
