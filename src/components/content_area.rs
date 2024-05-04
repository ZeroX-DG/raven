use std::sync::Arc;

use freya::prelude::*;

use crate::{
    events::{Event, Events},
    pane::Pane,
};

#[component]
#[allow(non_snake_case)]
pub fn ContentArea(
    // Pane to render the content of
    pane: Arc<Pane>,
    // Size of each cell (width, height)
    cell_size: (f32, f32),
) -> Element {
    let mut lines = use_signal_sync(|| vec![]);
    let mut cursor_position = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut scroll_top = use_signal_sync(|| 0);

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
        let pane = pane.clone();
        move |e: WheelEvent| {
            let delta_y = e.data.get_delta_y();
            pane.scroll(delta_y);
        }
    };

    use_memo({
        let pane = pane.clone();
        move || {
            let terminal_size = terminal_size();
            pane.resize(terminal_size, cell_size, line_spacing);
        }
    });

    use_hook(move || {
        let events = Events::get();
        events.subscribe(move |event| match event {
            Event::PaneOutput(pane_id) if pane_id == pane.id => {
                let rendered = pane.render();

                *lines.write() = rendered.lines;
                *cursor_position.write() = (rendered.cursor.x, rendered.cursor.y as usize);
                *scroll_top.write() = rendered.scroll_top;
            }
            _ => {}
        });
    });

    rsx!(
        rect {
            reference: node_ref,
            width: "100%",
            height: "100%",
            padding: "{padding_top} {padding_right} {padding_bottom} {padding_left}",
            onwheel: onwheel,
            for (line_index, line) in lines().iter().enumerate() {
                rect {
                    padding: "{line_spacing} 0",
                    paragraph {
                        max_lines: "1",
                        for segment in line.clusters() {
                            text { "{segment.text}" }
                        }
                    }
                    if line_index == cursor_position().1 && scroll_top() == 0 {
                        rect {
                            width: "{cell_size.0}",
                            height: "{cell_size.1}",
                            color: "rgb(17, 21, 28)",
                            background: "rgb(165, 172, 186)",
                            layer: "-10",
                            position: "absolute",
                            position_top: "0",
                            position_left: "{cell_size.0 * cursor_position().0 as f32}",

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
