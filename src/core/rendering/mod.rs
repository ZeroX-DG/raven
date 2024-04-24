use wezterm_term::Terminal;

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

pub fn render_terminal(terminal: &Terminal) -> Vec<LineElement> {
    let mut lines = vec![];

    terminal.screen().for_each_phys_line(|_, line| {
        let segments = line.cluster(None).iter().map(|cluster| {
            LineSegment {
                text: cluster.text.clone()
            }
        }).collect();
        lines.push(LineElement::new(segments));
    });

    lines
}