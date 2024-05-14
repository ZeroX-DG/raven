pub struct TerminalConfig {
    // Font size of the terminal
    pub font_size: f32,

    // Line height of the terminal.
    // By default it let the line height determined by the render engine.
    pub line_height: Option<f32>,
}

impl TerminalConfig {
    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_size: 14.,
            line_height: None,
        }
    }
}
