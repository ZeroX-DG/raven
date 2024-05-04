use std::sync::Arc;

use wezterm_term::TerminalSize;

use crate::pane::{alloc_pane_id, read_from_pane_pty, Pane, PaneId};

pub struct AppState {
    panes: Vec<Arc<Pane>>,
    active_pane_id: Option<PaneId>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            active_pane_id: None,
        }
    }

    pub fn active_pane(&self) -> Option<Arc<Pane>> {
        self.active_pane_id
            .map(|active_id| self.panes.iter().find(|pane| pane.id == active_id).cloned())
            .flatten()
    }

    pub fn set_active_pane(&mut self, pane_id: PaneId) {
        self.active_pane_id.replace(pane_id);
    }

    pub fn panes(&self) -> Vec<Arc<Pane>> {
        self.panes.clone()
    }

    pub fn get_pane(&self, id: PaneId) -> Option<Arc<Pane>> {
        self.panes.iter().find(|pane| pane.id == id).cloned()
    }

    pub fn new_pane(&mut self) -> Arc<Pane> {
        let pane_id = alloc_pane_id();

        let initial_rows = 24;
        let initial_cols = 80;

        let pane = Arc::new(
            Pane::new(
                pane_id,
                TerminalSize {
                    rows: initial_rows,
                    cols: initial_cols,
                    pixel_width: 0,
                    pixel_height: 0,
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
