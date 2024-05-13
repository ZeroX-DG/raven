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

    pub fn resize(&self, terminal_size: (f32, f32), cell_size: (f32, f32)) {
        let (terminal_width, terminal_height) = terminal_size;
        let (cell_width, cell_height) = cell_size;

        let cols = f32::max(terminal_width / cell_width, 1.) as usize;
        let rows = f32::max(terminal_height / cell_height, 1.) as usize;

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

    pub fn mouse_down(&self, event: PointerEvent, cell_size: (f32, f32)) {
        self.send_mouse_event(event, wezterm_term::MouseEventKind::Press, cell_size);
    }

    pub fn mouse_up(&self, event: PointerEvent, cell_size: (f32, f32)) {
        self.send_mouse_event(event, wezterm_term::MouseEventKind::Release, cell_size);
    }

    pub fn mouse_move(&self, event: PointerEvent, cell_size: (f32, f32)) {
        self.send_mouse_event(event, wezterm_term::MouseEventKind::Move, cell_size);
    }

    fn send_mouse_event(
        &self,
        event: PointerEvent,
        kind: wezterm_term::MouseEventKind,
        cell_size: (f32, f32),
    ) {
        let (cell_width, cell_height) = cell_size;
        let col = (event.element_coordinates.x / cell_width as f64) as usize;
        let row = (event.element_coordinates.y / cell_height as f64) as i64;
        let mouse_button = match event.get_pointer_type() {
            PointerType::Mouse { trigger_button } => match trigger_button {
                Some(MouseButton::Left) => wezterm_term::MouseButton::Left,
                Some(MouseButton::Right) => wezterm_term::MouseButton::Right,
                Some(MouseButton::Middle) => wezterm_term::MouseButton::Middle,
                _ => wezterm_term::MouseButton::None,
            },
            PointerType::Touch { .. } => wezterm_term::MouseButton::Left,
        };

        let event = wezterm_term::MouseEvent {
            kind,
            button: mouse_button,
            modifiers: wezterm_term::KeyModifiers::NONE,
            x: col,
            y: row,
            x_pixel_offset: event.element_coordinates.x as isize,
            y_pixel_offset: event.element_coordinates.y as isize,
        };
        self.send_event(UserEvent::Mouse(event));
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
