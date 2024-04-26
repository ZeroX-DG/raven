use std::sync::Arc;

use freya::prelude::*;

use crate::{core::{pane::Pane, rendering::render_terminal}, events::{Event, Events}, utils::get_cell_size};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(pane: Arc<Pane>) -> Element {
    let mut lines = use_signal_sync(|| vec![]);
    let mut cursor_position = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut cell_size = use_signal::<(f32, f32)>(|| (0., 0.));

    use_hook(move || {
        // Calculate cell size async to prevent blocking rendering
        spawn(async move {
            cell_size.set(get_cell_size(14.));
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
            padding: "50 50 20 100",
            for (line_index, line) in lines().iter().enumerate() {
                rect {
                    padding: "2 0",
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
