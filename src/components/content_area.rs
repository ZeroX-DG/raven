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
    let padding_bottom = 40.;
    let padding_left = 100.;
    let line_spacing: u16 = 2;

    let (node_ref, size) = use_node_signal();

    let terminal_size = use_memo(move || {
        let size = size.read();
        let width = f32::max(size.area.width() - (padding_left + padding_right), 0.);
        let height = f32::max(
            size.area.height()
                - (padding_top + padding_bottom)
                - (line_spacing as f32 * rendered_lines().len() as f32),
            0.,
        );
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
            let mut terminal_event_rx = pane.terminal_bridge().terminal_event_receiver();
            spawn(async move {
                while let Ok(event) = terminal_event_rx.recv().await {
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
                        TerminalEvent::Exit => {
                            pane.close();
                            break;
                        }
                    }
                }
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
                Line {
                    key: "{line_index}",
                    line: line.clone(),
                    line_spacing: line_spacing,
                    cursor: (line_index == rendered_cursor().1 && rendered_scroll_top() == 0).then(|| rendered_cursor().0),
                    cell_size: cell_size,
                }
            }
        }
    )
}

#[component]
#[allow(non_snake_case)]
fn Line(
    line: LineElement,
    line_spacing: u16,
    cursor: Option<usize>,
    cell_size: (f32, f32),
) -> Element {
    rsx!(
        CursorArea {
            icon: CursorIcon::Text,
            rect {
                onmousedown: |e| e.stop_propagation(),
                height: "{cell_size.1 + (2. * line_spacing as f32)}",
                for segment in line.clusters() {
                    rect {
                        background: segment.background(),
                        width: "{cell_size.0 * segment.width() as f32 + 0.5}",
                        height: "{cell_size.1 + (2. * line_spacing as f32)}",
                        layer: "2",
                        position: "absolute",
                        position_top: "0",
                        position_left: "{cell_size.0 * segment.start_index() as f32}",
                    }
                }
                rect {
                    padding: "{line_spacing} 0",
                    paragraph {
                        max_lines: "1",
                        layer: "1",
                        for segment in line.clusters() {
                            text {
                                color: segment.foreground(),
                                font_weight: "{segment.intensity()}",
                                "{segment.text()}"
                            }
                        }
                    }
                    if let Some(cursor_index) = cursor {
                        rect {
                            width: "{cell_size.0}",
                            height: "{cell_size.1}",
                            color: "rgb(17, 21, 28)",
                            background: "rgb(165, 172, 186)",
                            layer: "-10",
                            position: "absolute",
                            position_top: "0",
                            position_left: "{cell_size.0 * cursor_index as f32}",

                            rect {
                                label {
                                    "{line.cell_content(cursor_index)}"
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
