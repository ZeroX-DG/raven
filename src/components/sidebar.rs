use std::sync::Arc;

use freya::prelude::*;

use crate::core::pane::Pane;
use crate::icons::TERMINAL_ICON;

#[component]
#[allow(non_snake_case)]
pub fn Sidebar(panes: Vec<Arc<Pane>>) -> Element {
    rsx!(
        rect {
            padding: "50 50 20 30",
            label {
                "Workspace"
            }

            for pane in panes {
                paragraph {
                    margin: "8 0",
                    text { font_size: "12", color: "rgb(86, 91, 120, 0.6)", "{TERMINAL_ICON}" }
                    text { "  " }
                    text { color: "rgb(165, 172, 186)", "{pane.title()}" }
                }
            }
        }
    )
}
