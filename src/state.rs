use std::sync::{Arc, Mutex};

use wezterm_term::TerminalSize;

use crate::core::pane::{alloc_pane_id, read_from_pane_pty, Pane, PaneId};

pub struct AppState {
    panes: Mutex<Vec<Arc<Pane>>>,
    active_pane_id: Mutex<Option<PaneId>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            panes: Mutex::new(Vec::new()),
            active_pane_id: Mutex::new(None)
        }
    }

    pub fn active_pane(&self) -> Option<Arc<Pane>> {
        self.active_pane_id.lock().unwrap().map(|active_id| {
            self.panes.lock().unwrap().iter().find(|pane| pane.id == active_id).cloned()
        }).flatten()
    }

    pub fn set_active_pane(&self, pane_id: PaneId) {
        self.active_pane_id.lock().unwrap().replace(pane_id);
    }
    
    pub fn new_pane(&self) -> Arc<Pane> {
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

        self.panes.lock().unwrap().push(pane.clone());

        std::thread::spawn({
            let pane = Arc::clone(&pane);
            move || read_from_pane_pty(pane)
        });

        pane
    }
}