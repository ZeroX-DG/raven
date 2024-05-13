use termwiz::cellcluster::CellCluster;
use wezterm_term::{color::ColorPalette, CursorPosition, Line, Terminal};

#[derive(Clone, Debug)]
pub struct LineElement(Line, ColorPalette, usize);

#[derive(Clone, Debug)]
pub struct LineSegment(CellCluster, ColorPalette);

impl LineElement {
    pub fn clusters(&self) -> Vec<LineSegment> {
        let mut line = self.0.clone();
        let seq_no = line.current_seqno();
        let remaining_space = usize::max(self.2, line.len()) - line.len();
        let empty_space_line = Line::with_width(remaining_space, seq_no);
        line.append_line(empty_space_line, seq_no);

        line.cluster(None)
            .into_iter()
            .map(|cluster| LineSegment(cluster, self.1.clone()))
            .collect()
    }
}

impl LineSegment {
    pub fn is_bold(&self) -> bool {
        match self.0.attrs.intensity() {
            wezterm_term::Intensity::Bold => true,
            wezterm_term::Intensity::Half => true,
            _ => false,
        }
    }

    pub fn foreground(&self) -> (u8, u8, u8, u8) {
        let foreground = self.1.resolve_fg(self.0.attrs.foreground());
        foreground.as_rgba_u8()
    }

    pub fn background(&self) -> (u8, u8, u8, u8) {
        let background = self.1.resolve_bg(self.0.attrs.background());
        background.as_rgba_u8()
    }

    pub fn width(&self) -> usize {
        self.0.width
    }

    pub fn text(&self) -> String {
        self.0.text.clone()
    }
}

impl PartialEq for LineElement {
    fn eq(&self, other: &Self) -> bool {
        self.0.current_seqno() == other.0.current_seqno() && self.0.as_str() == other.0.as_str()
    }
}

pub fn render_terminal(
    terminal: &Terminal,
    scroll_top: usize,
) -> (Vec<LineElement>, CursorPosition) {
    let mut lines = vec![];

    let screen = terminal.screen();
    let first_visible_line_index = screen.scrollback_rows() - screen.physical_rows - scroll_top;
    let last_visible_line_index = first_visible_line_index + screen.physical_rows;
    let color_palette = terminal.get_config().color_palette();

    terminal.screen().for_each_phys_line(|index, line| {
        if index < first_visible_line_index || index > last_visible_line_index {
            return;
        }
        lines.push(LineElement(
            line.clone(),
            color_palette.clone(),
            screen.physical_cols,
        ));
    });

    let cursor_position = terminal.cursor_pos();

    (lines, cursor_position)
}
