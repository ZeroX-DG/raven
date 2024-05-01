use std::sync::Arc;

use freya::prelude::*;

use crate::{{pane::Pane, rendering::render_terminal}, events::{Event, Events}, utils::get_cell_size};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(pane: Arc<Pane>, font_size: f32) -> Element {
    let mut lines = use_signal_sync(|| vec![]);
    let mut cursor_position = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut cell_size = use_signal::<(f32, f32)>(|| (1., 1.));

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

    use_memo({
        let pane = pane.clone();
        move || {
            let terminal_size = terminal_size();
            pane.resize(terminal_size, *cell_size.read(), line_spacing);
        }
    });

    use_hook(move || {
        // Calculate cell size async to prevent blocking rendering
        spawn(async move {
            cell_size.set(get_cell_size(font_size));
        });
    });

    use_hook(move || {
        let events = Events::get();
        events.subscribe(move |event| {
            match event {
                Event::PaneOutput(pane_id) if pane_id == pane.id => {
                    let terminal = pane.terminal()
                        .lock()
                        .expect("Unable to obtain terminal");

                    let (rendered_lines, rendered_cursor_position) = render_terminal(&terminal);
                    lines.set(rendered_lines);
                    cursor_position.set((rendered_cursor_position.x, rendered_cursor_position.y as usize));
                }
                _ => {}
            }
        });
    });

    rsx!(
        rect {
            reference: node_ref,
            width: "100%",
            height: "100%",
            padding: "{padding_top} {padding_right} {padding_bottom} {padding_left}",
            for (line_index, line) in lines().iter().enumerate() {
                rect {
                    padding: "{line_spacing} 0",
                    paragraph {
                        max_lines: "1",
                        for segment in line.clusters() {
                            text { "{segment.text}" }
                        }
                    }
                    if line_index == cursor_position().1 {
                        rect {
                            width: "{cell_size().0}",
                            height: "{cell_size().1}",
                            color: "rgb(17, 21, 28)",
                            background: "rgb(165, 172, 186)",
                            layer: "-10",
                            position: "absolute",
                            position_top: "0",
                            position_left: "{cell_size().0 * cursor_position().0 as f32}",

                            rect {
                                label {
                                    "{line.cell_content(cursor_position().0)}"
                                }
                            }
                        }
                    }
                }
            }
        }
    )
}
