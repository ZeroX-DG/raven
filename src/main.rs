mod components;
mod hooks;

use freya::prelude::*;
use components::{Sidebar, ContentArea};
use hooks::use_terminal::use_terminal;

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMono-Regular.ttf");

fn main() {
    launch_cfg(App, LaunchConfig::<()>::builder()
        .with_decorations(false)
        .with_width(900.)
        .with_height(600.)
        .with_transparency(true)
        .with_font("jetbrains mono", JETBRAINS_MONO)
        .with_default_font("jetbrains mono")
        .build());
}

#[component]
#[allow(non_snake_case)]
fn App() -> Element {
    let terminal = use_terminal();

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(17, 21, 28)",
            color: "rgb(86, 91, 120)",
            direction: "horizontal",
            font_size: "14",
            Sidebar {}
            ContentArea {
                lines: terminal.active_session_lines()
            }
        }
    )
}
