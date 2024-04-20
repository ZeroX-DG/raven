use freya::prelude::*;

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
    let content = vec![
        "-> raven git:(master) cargo run".to_string(),
        "   Compiling raven v0.1.0 (/some/path/raven)".to_string(),
        "    Finished dev [unoptimized + debuginfo] target(s) in 0.93s".to_string(),
        "     Running `target/debug/raven`".to_string()
    ];
    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(17, 21, 28)",
            color: "rgb(86, 91, 120)",
            direction: "horizontal",
            font_size: "14",
            Sidebar {}
            Content {
                lines: content
            }
        }
    )
}

#[component]
#[allow(non_snake_case)]
fn Sidebar() -> Element {
    rsx!(
        rect {
            padding: "50 50 20 30",
            label {
                "Workspace"
            }
        }
    )
}

#[component]
#[allow(non_snake_case)]
fn Content(lines: Vec<String>) -> Element {
    rsx!(
        rect {
            padding: "50 50 20 100",
            for line in lines {
                rect {
                    padding: "2 0", 
                    label { "{line}" }
                }
            }
        }
    )
}