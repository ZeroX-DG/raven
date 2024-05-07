use termwiz::{cellcluster::CellCluster, color::SrgbaTuple};
use wezterm_term::{color::ColorPalette, CellRef, CursorPosition, Line, Terminal};

#[derive(Clone, Debug)]
pub struct LineElement(Line, ColorPalette, usize);

#[derive(Clone, Debug)]
pub struct LineSegment(CellCluster, ColorPalette);

impl LineElement {
    pub fn clusters(&self) -> Vec<LineSegment> {
        let mut line = self.0.clone();
        let seq_no = line.current_seqno();
        let remaining_space = self.2 - line.len();
        let empty_space_line = Line::with_width(remaining_space, seq_no);
        line.append_line(empty_space_line, seq_no);

        line.cluster(None)
            .into_iter()
            .map(|cluster| LineSegment(cluster, self.1.clone()))
            .collect()
    }

    pub fn cell(&self, index: usize) -> Option<CellRef> {
        self.0.get_cell(index)
    }

    pub fn cell_content(&self, index: usize) -> String {
        self.cell(index)
            .map(|cell| cell.str().to_string())
            .unwrap_or_default()
    }
}

impl LineSegment {
    pub fn intensity(&self) -> &'static str {
        match self.0.attrs.intensity() {
            wezterm_term::Intensity::Bold => "bold",
            wezterm_term::Intensity::Half => "semi-bold",
            _ => "normal",
        }
    }

    pub fn foreground(&self) -> String {
        let foreground = self.1.resolve_fg(self.0.attrs.foreground());
        srgba_tuple_to_rgba(foreground)
    }

    pub fn background(&self) -> String {
        let background = self.1.resolve_bg(self.0.attrs.background());
        srgba_tuple_to_rgba(background)
    }

    pub fn width(&self) -> usize {
        self.0.width
    }

    pub fn text(&self) -> String {
        self.0.text.clone()
    }

    pub fn start_index(&self) -> usize {
        self.0.first_cell_idx
    }
}

fn srgba_tuple_to_rgba(color: SrgbaTuple) -> String {
    format!(
        "rgb({}, {}, {}, {})",
        color.0 * 255.,
        color.1 * 255.,
        color.2 * 255.,
        color.3 * 255.,
    )
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
    let color_palette = terminal.get_config().color_palette();

    terminal.screen().for_each_phys_line(|index, line| {
        if index < first_visible_line_index {
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
