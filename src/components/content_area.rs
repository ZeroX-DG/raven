use std::sync::Arc;

use freya::prelude::*;
use skia_safe::{Color, Font, FontStyle, Paint};

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
    // Size of the font
    font_size: f32,
) -> Element {
    let mut rendered_lines = use_signal_sync::<Vec<LineElement>>(|| vec![]);
    let mut rendered_cursor = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut rendered_scroll_top = use_signal_sync::<usize>(|| 0);
    let terminal = use_terminal(pane.clone());

    let padding_top = 50.;
    let padding_right = 50.;
    let padding_bottom = 40.;
    let padding_left = 100.;
    let line_spacing = 2.;

    let (node_ref, size) = use_node_signal();

    let terminal_size = use_memo(move || {
        let size = size.read();
        let width = f32::max(size.area.width(), 0.);
        let height = f32::max(size.area.height(), 0.);
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
        let terminal_event_rx = pane.terminal_bridge().terminal_event_receiver().clone();
        move || {
            spawn(async move {
                while let Ok(event) = terminal_event_rx.recv_async().await {
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

    let canvas = use_canvas(
        (&*rendered_lines.read(), &*rendered_cursor.read()),
        move |(lines, cursor)| {
            Box::new(move |canvas, font_collection, region| {
                if lines.len() == 0 {
                    return;
                }

                canvas.translate((region.min_x(), region.min_y()));
                canvas.scale((2., 2.));

                let normal_typeface =
                    font_collection.find_typefaces(&["jetbrains mono"], FontStyle::default());

                let bold_typeface =
                    font_collection.find_typefaces(&["jetbrains mono"], FontStyle::bold());

                let normal_font = Font::new(
                    normal_typeface
                        .first()
                        .expect("JetBrains Mono Font not found"),
                    font_size,
                );

                let bold_font = Font::new(
                    bold_typeface
                        .first()
                        .expect("JetBrains Mono Font not found"),
                    font_size,
                );

                let draw_text = |content: &str, x: f32, y: f32, color: Color, bold: bool| {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(color);
                    canvas.draw_str(
                        content,
                        (x, y + cell_size.1),
                        if bold { &bold_font } else { &normal_font },
                        &paint,
                    )
                };

                let draw_rect = |x: f32, y: f32, width: f32, height: f32, color: Color| {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(color);
                    canvas.draw_rect(skia_safe::Rect::from_xywh(x, y, width, height), &paint);
                };

                let mut x = 0.;
                let mut y = line_spacing;

                let mut cursor_y = y + line_spacing;

                for (line_index, line) in lines.iter().enumerate() {
                    if line_index == cursor.1 {
                        cursor_y = y;
                    }

                    for cluster in line.clusters() {
                        let background = cluster.background();
                        let background = Color::from_rgb(background.0, background.1, background.2);
                        draw_rect(
                            x,
                            y + line_spacing,
                            cell_size.0 * cluster.width() as f32,
                            cell_size.1 + line_spacing,
                            background,
                        );
                        x += cell_size.0 * cluster.width() as f32;
                    }
                    x = 0.;
                    for cluster in line.clusters() {
                        let foreground = cluster.foreground();
                        let foreground = Color::from_rgb(foreground.0, foreground.1, foreground.2);

                        draw_text(&cluster.text(), x, y, foreground, cluster.is_bold());
                        x += cell_size.0 * cluster.width() as f32;
                    }
                    x = 0.;
                    y += cell_size.1 + line_spacing;
                }

                // draw the cursor at the end so it sits on top everything
                draw_rect(
                    cursor.0 as f32 * cell_size.0,
                    cursor_y + line_spacing * 2.,
                    cell_size.0,
                    cell_size.1,
                    Color::WHITE,
                );
                draw_text(
                    &lines.get(cursor.1).unwrap().cell_content(cursor.0),
                    cursor.0 as f32 * cell_size.0,
                    cursor_y,
                    Color::BLACK,
                    false,
                );
            })
        },
    );

    rsx!(
        rect {
            width: "100%",
            height: "100%",
            padding: "{padding_top} {padding_right} {padding_bottom} {padding_left}",
            onwheel: onwheel,
            rect {
                width: "100%",
                height: "100%",
                reference: node_ref,
                Canvas {
                    canvas,
                    theme: theme_with!(CanvasTheme {
                        background: "transparent".into(),
                        width: "100%".into(),
                        height: "100%".into(),
                    })
                }
            }
        }
    )
}
