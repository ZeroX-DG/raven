pub struct TerminalConfig {
    pub font_size: f32,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_size: 14.
        }
    }
}