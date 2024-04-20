use freya::prelude::*;

fn main() {
    launch_with_title(app, "Raven");
}

fn app() -> Element {
    rsx!(
        rect {
            width: "100%",
            height: "100%",
            label {
                "Raven Terminal Emulator"
            }
        }
    )
}
