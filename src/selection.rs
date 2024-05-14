use skia_safe::Rect;
use termwiz::surface::SequenceNo;

#[derive(Clone, PartialEq, Debug)]
pub struct Selection {
    pub seqno: SequenceNo,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

pub struct SelectionRange {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Selection {
    pub fn range(&self) -> SelectionRange {
        let is_reverse_selection = (self.start.0 > self.end.0 && self.start.1 == self.end.1)
            || (self.start.1 > self.end.1);

        let range_start = if is_reverse_selection {
            self.end
        } else {
            self.start
        };
        let mut range_end = if is_reverse_selection {
            self.start
        } else {
            self.end
        };

        // Shift by one for the offset error in reverse range.
        if is_reverse_selection {
            range_end.0 += 1;
        }

        SelectionRange {
            start: range_start,
            end: range_end,
        }
    }

    pub fn get_content(&self, terminal: &wezterm_term::Terminal) -> String {
        let screen = terminal.screen();
        let mut content = String::new();
        let selection_range = self.range();

        let num_of_rows = (selection_range.end.1 - selection_range.start.1) + 1;

        let mut x = selection_range.start.0;
        let mut y = selection_range.start.1;

        screen.for_each_phys_line(|line_index, line| {
            if line_index < selection_range.start.1 || line_index > selection_range.end.1 {
                return;
            }

            let is_last_line = y - selection_range.start.1 == num_of_rows - 1;

            let line_end = if num_of_rows == 1 || is_last_line {
                selection_range.end.0
            } else {
                line.len()
            };

            content.push_str(&line.columns_as_str(x..line_end));
            content.push('\n');
            x = 0;
            y += 1;
        });

        content
    }

    pub fn render(
        &self,
        first_line_index: usize,
        cell_size: (f32, f32),
        terminal_size: (usize, usize),
    ) -> Vec<Rect> {
        let mut rects = Vec::new();

        let (cell_width, cell_height) = cell_size;
        let (terminal_cols, _) = terminal_size;

        let range = self.range();
        let (col_start, mut line_start) = range.start;
        let (col_end, mut line_end) = range.end;

        line_start -= first_line_index;
        line_end -= first_line_index;

        if col_start == col_end && line_start == line_end {
            return rects;
        }

        let num_of_rows = (line_end - line_start) + 1;

        rects.push(Rect::from_xywh(
            col_start as f32 * cell_width,
            line_start as f32 * cell_height,
            if num_of_rows > 1 {
                cell_width * (terminal_cols - col_start) as f32
            } else {
                cell_width * (col_end - col_start) as f32
            },
            cell_height,
        ));

        if num_of_rows > 2 {
            for line_offset in 1..num_of_rows - 1 {
                rects.push(Rect::from_xywh(
                    0.,
                    (line_start + line_offset) as f32 * cell_height,
                    terminal_cols as f32 * cell_width,
                    cell_height,
                ));
            }
        }

        if num_of_rows > 1 {
            rects.push(Rect::from_xywh(
                0.,
                cell_height * line_end as f32,
                cell_width * col_end as f32,
                cell_height,
            ));
        }

        rects
    }
}
