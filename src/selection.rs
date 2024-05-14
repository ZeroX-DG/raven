use skia_safe::Rect;
use termwiz::surface::SequenceNo;

#[derive(Clone, PartialEq, Debug)]
pub struct Selection {
    pub seqno: SequenceNo,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl Selection {
    pub fn render(&self, cell_size: (f32, f32), terminal_size: (usize, usize)) -> Vec<Rect> {
        let mut rects = Vec::new();

        let (col_start, line_start) = self.start;
        let (col_end, line_end) = self.end;
        let (cell_width, cell_height) = cell_size;
        let (terminal_cols, _) = terminal_size;

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
