use serde::Deserialize;

#[derive(Deserialize)]
pub struct TerminalConfig {
    // Font size of the terminal
    #[serde(default = "default_font_size")]
    pub font_size: f32,

    // Line height of the terminal.
    // By default it let the line height determined by the render engine.
    pub line_height: Option<f32>,
}

impl TerminalConfig {
    pub fn load_from_file(&mut self) {
        let Some(config_dir) = dirs::config_dir() else {
            log::info!("Unable to find config dir. Not loading config...");
            return;
        };

        let config_file = config_dir.join("raven").join("config.toml");

        let content = match std::fs::read_to_string(config_file) {
            Ok(content) => content,
            Err(e) => {
                log::info!(
                    "Unable to read config file content. Not loading config...\n{}",
                    e.to_string()
                );
                return;
            }
        };

        let config = match toml::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                log::info!(
                    "Unable to parse config file. Not loading config...\n{}",
                    e.to_string()
                );
                return;
            }
        };

        *self = config;
    }

    pub fn set_font_size(&mut self, font_size: f32) {
        self.font_size = font_size;
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            line_height: None,
        }
    }
}

fn default_font_size() -> f32 {
    14.
}
