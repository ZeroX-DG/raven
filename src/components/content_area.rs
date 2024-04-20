use std::ops::Deref;

use freya::prelude::*;
use crate::hooks::use_terminal::Session;

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(active_session: Memo<Option<Session>>) -> Element {
    let lines = use_memo(move || {
        let maybe_session = active_session.read();
        maybe_session
            .deref()
            .clone()
            .map(|session| vec!["string"]).unwrap_or(Vec::new())
    });

    rsx!(
        rect {
            padding: "50 50 20 100",
            for line in lines() {
                rect {
                    padding: "2 0", 
                    label { "{line}" }
                }
            }
        }
    )
}