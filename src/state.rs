use std::sync::Arc;

use wezterm_term::TerminalSize;

use crate::core::pane::{alloc_pane_id, read_from_pane_pty, Pane, PaneId};

pub struct AppState {
    panes: Vec<Arc<Pane>>,
    active_pane_id: Option<PaneId>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            active_pane_id: None
        }
    }

    pub fn active_pane(&self) -> Option<Arc<Pane>> {
        self.active_pane_id.map(|active_id| {
            self.panes.iter().find(|pane| pane.id == active_id).cloned()
        }).flatten()
    }

    pub fn set_active_pane(&mut self, pane_id: PaneId) {
        self.active_pane_id.replace(pane_id);
    }
    
    pub fn new_pane(&mut self) -> Arc<Pane> {
        let pane_id = alloc_pane_id();
        let pane = Arc::new(
            Pane::new(
                pane_id,
                TerminalSize {
                    rows: 24,
                    cols: 80,
                    pixel_width: 100,
                    pixel_height: 100,
                    dpi: 1,
                },
            )
            .unwrap(),
        );

        self.panes.push(pane.clone());

        std::thread::spawn({
            let pane = Arc::clone(&pane);
            move || read_from_pane_pty(pane)
        });

        pane
    }
}