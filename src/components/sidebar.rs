use freya::prelude::*;

#[component]
#[allow(non_snake_case)]
pub fn Sidebar() -> Element {
    rsx!(
        rect {
            padding: "50 50 20 30",
            label {
                "Workspace"
            }
        }
    )
}