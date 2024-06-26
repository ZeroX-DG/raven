mod components;
mod config;
mod hooks;
mod icons;
mod pane;
mod rendering;
mod selection;
mod state;
mod terminal_loop;
mod utils;

use arboard::Clipboard;
use components::{ContentArea, Sidebar};
use config::TerminalConfig;
use freya::prelude::*;
use hooks::use_terminal;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use state::AppState;
use wezterm_term::{KeyCode, KeyModifiers};

const JETBRAINS_MONO: &[u8] = include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf");

fn main() {
    let config = ConfigBuilder::new()
        .add_filter_ignore_str("wezterm_term")
        .set_target_level(LevelFilter::Info)
        .build();

    TermLogger::init(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .ok();

    launch_cfg(
        App,
        LaunchConfig::<()>::new()
            .with_decorations(false)
            .with_width(900.)
            .with_height(600.)
            .with_transparency(true)
            .without_default_fonts()
            .with_font("jetbrains mono", JETBRAINS_MONO)
            .with_default_font("jetbrains mono"),
    );
}

#[component]
#[allow(non_snake_case)]
fn App() -> Element {
    let state = use_signal(|| {
        let mut state = AppState::new();
        let pane = state.new_pane();
        state.set_active_pane(pane.id);
        state
    });
    let active_pane = use_memo(move || state.read().active_pane());
    let mut focus_manager = use_focus();

    let mut config = use_signal(|| {
        let mut config = TerminalConfig::default();
        config.load_from_file();
        config
    });

    let onkeydown = move |e: KeyboardEvent| {
        focus_manager.prevent_navigation();
        let Some(pane) = active_pane.read().clone() else {
            return;
        };

        let terminal = use_terminal(pane);

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
                    terminal.paste(content);
                    return;
                };

                // Handle copying selection content from terminal
                if ch == "c" && meta_or_ctrl {
                    terminal.copy_selection();
                    return;
                };

                // Handle zoom in
                if (ch == "+" || ch == "=") && (meta_or_ctrl || e.modifiers.ctrl()) {
                    let new_font_size = config.read().font_size + 1.;
                    config.write().set_font_size(new_font_size);
                    return;
                }

                // Handle zoom out
                if (ch == "-" || ch == "_") && (meta_or_ctrl || e.modifiers.ctrl()) {
                    let new_font_size = config.read().font_size - 1.;
                    config.write().set_font_size(new_font_size);
                    return;
                }

                // Handle typing regular keys
                let key_code = KeyCode::Char(ch.chars().next().unwrap());
                terminal.key_down(key_code, mods);
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
                    terminal.key_down(key_code, mods);
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

                if let Some(pane) = active_pane() {
                    ContentArea {
                        pane: pane,
                        config: config
                    }
                }
            }
        }
    )
}
