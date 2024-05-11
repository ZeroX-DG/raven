use std::sync::Arc;

use freya::prelude::*;
use skia_safe::textlayout::{ParagraphBuilder, ParagraphStyle, TextStyle};
use skia_safe::{Color, Paint};

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
    // Line height
    line_height: Option<f32>,
) -> Element {
    let mut rendered_lines = use_signal_sync::<Vec<LineElement>>(|| vec![]);
    let mut rendered_cursor = use_signal_sync::<(usize, usize)>(|| (0, 0));
    let mut rendered_scroll_top = use_signal_sync::<usize>(|| 0);
    let terminal = use_terminal(pane.clone());

    let padding_top = 50.;
    let padding_right = 50.;
    let padding_bottom = 40.;
    let padding_left = 100.;

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
            terminal.resize(terminal_size, cell_size);
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

                let mut style = ParagraphStyle::default();
                let mut text_style = TextStyle::default();
                text_style.set_font_size(font_size);
                text_style.set_font_families(&["jetbrains mono"]);

                if let Some(line_height) = line_height {
                    text_style.set_height_override(true);
                    text_style.set_height(line_height);
                }

                style.set_text_style(&text_style);
                let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection.clone());

                let mut paint = Paint::default();
                paint.set_anti_alias(true);

                let mut y = 0.;
                let mut cursor_y = y;

                for (line_index, line) in lines.iter().enumerate() {
                    if line_index == cursor.1 {
                        cursor_y = y;
                    }

                    let clusters = &line.clusters();

                    for cluster in clusters {
                        let foreground = cluster.foreground();
                        let foreground = Color::from_rgb(foreground.0, foreground.1, foreground.2);

                        text_style.set_color(foreground);

                        if cluster.is_bold() {
                            text_style.set_font_style(skia_safe::FontStyle::bold());
                        } else {
                            text_style.set_font_style(skia_safe::FontStyle::normal());
                        }

                        paragraph_builder.push_style(&text_style);
                        paragraph_builder.add_text(cluster.text());
                    }

                    let mut paragraph = paragraph_builder.build();
                    paragraph.layout(skia_safe::scalar::MAX);

                    let mut x = 0.;
                    for cluster in clusters {
                        let background = cluster.background();
                        let background = Color::from_rgb(background.0, background.1, background.2);
                        let cluster_width = cell_size.0 * cluster.width() as f32;
                        paint.set_color(background);

                        canvas.draw_rect(
                            skia_safe::Rect::from_xywh(x, y, cluster_width, paragraph.height()),
                            &paint,
                        );
                        x += cluster_width;
                    }

                    paragraph.paint(canvas, (0., y));

                    paragraph_builder.reset();

                    y += paragraph.height();
                }

                // draw the cursor at the end so it sits on top everything
                let mut paint = Paint::default();
                paint.set_anti_alias(true);
                paint.set_color(Color::WHITE);
                paint.set_blend_mode(skia_safe::BlendMode::Difference);
                canvas.draw_rect(
                    skia_safe::Rect::from_xywh(
                        cursor.0 as f32 * cell_size.0,
                        cursor_y,
                        cell_size.0,
                        cell_size.1,
                    ),
                    &paint,
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
