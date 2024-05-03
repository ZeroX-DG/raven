mod components;
mod pane;
mod rendering;
mod state;
mod events;
mod icons;
mod utils;
mod config;

use components::{ContentArea, Sidebar};
use freya::prelude::*;
use state::AppState;
use events::{Event, Events};
use wezterm_term::{KeyCode, KeyModifiers};
use arboard::Clipboard;
use config::TerminalConfig;
use utils::get_cell_size;

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf");

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
    let mut focus_manager = use_focus();

    let config = use_signal(|| TerminalConfig::default());

    let cell_size = use_memo(move || {
        let font_size = config.read().font_size;
        get_cell_size(font_size)
    });

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

    let onkeydown = move |e: KeyboardEvent| {
        focus_manager.prevent_navigation();
        let Some(pane) = active_pane.read().clone() else {
            return
        };

        let terminal = pane.terminal();

        let mods = if e.modifiers.alt() {
            KeyModifiers::ALT
        } else if e.modifiers.shift() {
            KeyModifiers::SHIFT
        } else if e.modifiers.meta() {
            KeyModifiers::SUPER
        } else if e.modifiers.ctrl() {
            KeyModifiers::CTRL
        } else {
            KeyModifiers::NONE
        };

        match &e.key {
            Key::Character(ch) => {
                let meta_or_ctrl = if cfg!(target_os = "macos") {
                    e.modifiers.meta()
                } else {
                    e.modifiers.ctrl()
                };

                // Handle pasting content to terminal
                if ch == "v" && meta_or_ctrl {
                    let mut clipboard = Clipboard::new().unwrap();
                    let content = clipboard.get_text().unwrap();
                    terminal.lock().unwrap().send_paste(&content).unwrap();
                    return;
                };

                // Handle typing regular keys
                let keycode = KeyCode::Char(ch.chars().next().unwrap());
                terminal.lock().unwrap().key_down(keycode, mods).unwrap();
            }
            key => {
                let recognised_key = match key {
                    Key::Enter => Some(KeyCode::Enter),
                    Key::Backspace => Some(KeyCode::Backspace),
                    Key::Tab => Some(KeyCode::Tab),
                    Key::ArrowDown => Some(KeyCode::DownArrow),
                    Key::ArrowLeft => Some(KeyCode::LeftArrow),
                    Key::ArrowRight => Some(KeyCode::RightArrow),
                    Key::ArrowUp => Some(KeyCode::UpArrow),
                    Key::Shift => Some(KeyCode::Shift),
                    Key::Control => Some(KeyCode::Control),
                    Key::Escape => Some(KeyCode::Escape),
                    Key::Super => Some(KeyCode::Super),
                    key => {
                        println!("Unrecognised key: {}", key);
                        None
                    }
                };

                if let Some(key_code) = recognised_key {
                    terminal.lock().unwrap().key_down(key_code, mods).unwrap();
                }
            }
        };
    };

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            background: "rgb(17, 21, 28)",
            color: "rgb(86, 91, 120)",
            direction: "horizontal",
            font_size: "{config.read().font_size}",
            onkeydown: onkeydown,
            Sidebar {
                panes: state.read().panes()
            }

            rect {
                width: "calc(100% - 250)",
                height: "100%",

                WindowDragArea {
                    if let Some(pane) = active_pane() {
                        ContentArea {
                            pane: pane,
                            cell_size: cell_size()
                        }
                    }
                }
            }
        }
    )
}
