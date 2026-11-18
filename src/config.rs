// User configuration loader & serializer for Winscroller 2d Appview
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub columns: usize,
    pub rows: usize,
    pub icon_size: f32,
    pub show_labels: bool,
    pub backdrop: String, // "Wallpaper", "Glass", "Acrylic", "Dark", "Solid"
    pub centered: bool,
    pub card_width: f32,  // 0.3 - 1.0 fraction of screen
    pub card_height: f32, // 0.3 - 1.0 fraction of screen
    pub glow: bool,
    pub accent_color: String,
    pub hotkey: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            columns: 7,
            rows: 5,
            icon_size: 76.0,
            show_labels: true,
            backdrop: "Wallpaper".to_string(),
            centered: false,
            card_width: 0.75,
            card_height: 0.80,
            glow: true,
            accent_color: "#89b4fa".to_string(), // Soft pastel blue / Catppuccin
            hotkey: "Alt+Space".to_string(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        if let Some(app_data) = dirs::config_dir() {
            let dir = app_data.join("Winscroller2dAppview");
            let _ = fs::create_dir_all(&dir);
            dir.join("config.toml")
        } else {
            PathBuf::from("config.toml")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str::<Config>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Ok(content) = toml::to_string_pretty(self) {
            fs::write(path, content)?;
        }
        Ok(())
    }
}
