use std::sync::Arc;

use freya::prelude::*;

use crate::{
    hooks::use_terminal, pane::Pane, rendering::LineElement, terminal_loop::TerminalEvent,
};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(
    // Pane to render the content of
    pane: Arc<Pane>,
    // Size of each cell (width, height)
    cell_size: (f32, f32),
) -> Element {
    let mut rendered_lines = use_signal_sync::<Vec<LineElement>>(|| vec![]);
    let mut rendered_cursor = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut rendered_scroll_top = use_signal_sync::<usize>(|| 0);
    let terminal = use_terminal(pane.clone());

    let padding_top = 50.;
    let padding_right = 50.;
    let padding_bottom = 50.;
    let padding_left = 100.;
    let line_spacing = 2;

    let (node_ref, size) = use_node_signal();

    let terminal_size = use_memo(move || {
        let size = size.read();
        let width = f32::max(size.area.width() - (padding_left + padding_right), 0.);
        let height = f32::max(size.area.height() - (padding_top + padding_bottom), 0.);
        (width, height)
    });

    let onwheel = {
        let terminal = terminal.clone();
        move |e: WheelEvent| {
            let delta_y = e.data.get_delta_y();
            terminal.scroll(delta_y);
        }
    };

    use_memo({
        let terminal = terminal.clone();
        move || {
            let terminal_size = terminal_size();
            terminal.resize(terminal_size, cell_size, line_spacing);
        }
    });

    use_hook({
        let pane = pane.clone();
        move || {
            let terminal_event_rx = pane.terminal_events();
            spawn(async move {
                let _ = tokio::spawn(async move {
                    while let Ok(event) = terminal_event_rx.recv() {
                        match event {
                            TerminalEvent::Redraw {
                                lines,
                                cursor,
                                scroll_top,
                            } => {
                                *rendered_lines.write() = lines;
                                *rendered_cursor.write() = (cursor.x, cursor.y as usize);
                                *rendered_scroll_top.write() = scroll_top;
                            }
                        }
                    }
                })
                .await;
            });
        }
    });

    rsx!(
        rect {
            reference: node_ref,
            width: "100%",
            height: "100%",
            padding: "{padding_top} {padding_right} {padding_bottom} {padding_left}",
            onwheel: onwheel,
            for (line_index, line) in rendered_lines().iter().enumerate() {
                CursorArea {
                    icon: CursorIcon::Text,
                    rect {
                        padding: "{line_spacing} 0",
                        onmousedown: |e| e.stop_propagation(),
                        paragraph {
                            max_lines: "1",
                            for segment in line.clusters() {
                                text { "{segment.text}" }
                            }
                        }
                        if line_index == rendered_cursor().1 && rendered_scroll_top() == 0 {
                            rect {
                                width: "{cell_size.0}",
                                height: "{cell_size.1}",
                                color: "rgb(17, 21, 28)",
                                background: "rgb(165, 172, 186)",
                                layer: "-10",
                                position: "absolute",
                                position_top: "0",
                                position_left: "{cell_size.0 * rendered_cursor().0 as f32}",

                                rect {
                                    label {
                                        "{line.cell_content(rendered_cursor().0)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
