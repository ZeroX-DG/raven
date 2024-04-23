use std::sync::{mpsc::channel, Arc};

use freya::prelude::*;
use wezterm_term::TerminalSize;

use crate::core::{
    pane::{read_from_pane_pty, Pane},
    rendering::{render_terminal, LineElement},
};

pub struct UseTerminal {
    active_session_lines: SyncSignal<Vec<LineElement>>,
}

impl UseTerminal {
    pub fn active_session_lines(&self) -> SyncSignal<Vec<LineElement>> {
        self.active_session_lines
    }
}

pub fn use_terminal() -> UseTerminal {
    let mut active_session_lines = use_signal_sync::<Vec<LineElement>>(|| Vec::new());

    use_hook(|| {
        let pane = Arc::new(
            Pane::new(
                0,
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

        std::thread::spawn(move || {
            let (update_tx, update_rx) = channel();

            std::thread::spawn({
                let pane = Arc::clone(&pane);
                move || read_from_pane_pty(pane, update_tx)
            });

            loop {
                update_rx.recv().expect("Read thread died");
                let terminal = pane.terminal().lock().expect("Unable to obtain terminal");
                active_session_lines.set(render_terminal(&terminal));
            }

            // TODO Clean up after thread die
        });
    });

    UseTerminal {
        active_session_lines,
    }
}
