use std::rc::Rc;

use freya::prelude::*;
use crate::hooks::use_terminal::{Change, Session};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(active_session: Memo<Option<Rc<Session>>>) -> Element {
    let mut lines = use_signal_sync::<Vec<String>>(|| vec![]);

    use_hook(move || {
        let Some(session) = active_session() else {
            return;
        };
        let reader = session.reader.clone();
        std::thread::spawn(move || loop {
            let data = reader.lock().unwrap().recv().unwrap();
            match data {
                Change::Text(line) => {
                    lines.push(line);
                }
            }
        });
    });

    let write = move |_| {
        if let Some(session) = active_session() {
            session.write("ls\n");
        }
    };

    rsx!(
        rect {
            padding: "50 50 20 100",
            for line in lines() {
                rect {
                    padding: "2 0", 
                    label { onclick: write, "{line}" }
                }
            }
        }
    )
}