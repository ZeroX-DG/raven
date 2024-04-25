use wezterm_term::{CursorPosition, Terminal};

#[derive(Clone, Debug)]
pub struct LineElement {
    segments: Vec<LineSegment>
}

#[derive(Clone, Debug)]
pub struct LineSegment {
    pub text: String
}

impl LineElement {
    pub fn new(segments: Vec<LineSegment>) -> Self {
        Self {
            segments
        }
    }
    pub fn segments(&self) -> &Vec<LineSegment> {
        &self.segments
    }
}

pub fn render_terminal(terminal: &Terminal) -> (Vec<LineElement>, CursorPosition) {
    let mut lines = vec![];

    let screen = terminal.screen();
    let first_visible_line_index = screen.scrollback_rows() - screen.physical_rows;

    terminal.screen().for_each_phys_line(|index, line| {
        if index < first_visible_line_index {
            return;
        }
        let segments = line.cluster(None).iter().map(|cluster| {
            LineSegment {
                text: cluster.text.clone()
            }
        }).collect();
        lines.push(LineElement::new(segments));
    });

    let cursor_position = terminal.cursor_pos();

    (lines, cursor_position)
}