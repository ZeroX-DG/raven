mod components;
mod core;
mod state;
mod events;

use components::{ContentArea, Sidebar};
use freya::prelude::*;
use state::AppState;

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMonoNerdFontPropo-Regular.ttf");

fn main() {
    launch_cfg(
        App,
        LaunchConfig::<()>::builder()
            .with_decorations(false)
            .with_width(900.)
            .with_height(600.)
            .with_transparency(true)
            .with_font("jetbrains mono", JETBRAINS_MONO)
            .with_default_font("jetbrains mono")
            .build(),
    );
}

#[component]
#[allow(non_snake_case)]
fn App() -> Element {
    let state = use_signal(|| {
        let state = AppState::new();
        let pane = state.new_pane();
        state.set_active_pane(pane.id);
        state
    });
    let active_pane = use_memo(move || state.read().active_pane());

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(17, 21, 28)",
            color: "rgb(86, 91, 120)",
            direction: "horizontal",
            font_size: "14",
            Sidebar {}

            if let Some(pane) = active_pane() {
                ContentArea {
                    pane: pane
                }
            }
        }
    )
}
