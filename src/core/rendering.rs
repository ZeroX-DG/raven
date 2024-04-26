use termwiz::cellcluster::CellCluster;
use wezterm_term::{CursorPosition, Line, Terminal};

#[derive(Clone, Debug)]
pub struct LineElement(Line);

impl LineElement {
    pub fn clusters(&self) -> Vec<CellCluster> {
        self.0.cluster(None)
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
        lines.push(LineElement(line.clone()));
    });

    let cursor_position = terminal.cursor_pos();

    (lines, cursor_position)
}