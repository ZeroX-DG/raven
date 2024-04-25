mod components;
mod core;
mod state;
mod events;
mod icons;

use components::{ContentArea, Sidebar};
use freya::prelude::*;
use state::AppState;
use events::{Event, Events};

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
    let state = use_signal_sync(|| {
        let mut state = AppState::new();
        let pane = state.new_pane();
        state.set_active_pane(pane.id);
        state
    });
    let active_pane = use_memo(move || state.read().active_pane());

    use_hook(|| {
        let events = Events::get();
        events.subscribe(move |event| match event {
            Event::PaneTitle { pane_id, title } => {
                let pane = state.read().get_pane(pane_id);
                if let Some(pane) = pane {
                    pane.set_title(title);
                }
            }
            _ => {}
        });
    });

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(17, 21, 28)",
            color: "rgb(86, 91, 120)",
            direction: "horizontal",
            font_size: "14",
            Sidebar {
                panes: state.read().panes()
            }

            if let Some(pane) = active_pane() {
                ContentArea {
                    pane: pane
                }
            }
        }
    )
}
