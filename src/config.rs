// User configuration loader & serializer for Winscroller 2d Appview
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub columns: usize,
    pub rows: usize,
    pub icon_size: f32,
    pub show_labels: bool,
    pub backdrop: String, // "Wallpaper", "Centered", "Dark", "Solid"
    pub centered: bool,
    pub card_width: f32,  // 0.4 - 0.95 fraction of screen
    pub card_height: f32, // 0.4 - 0.95 fraction of screen
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

    #[allow(dead_code)]
    pub fn load() -> Self {
        Self::load_from(&Self::config_path())
    }

    pub fn load_from(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = toml::from_str::<Config>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save_to(path);
        default_cfg
    }

    #[allow(dead_code)]
    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_to(&Self::config_path())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            fs::write(path, content)?;
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.columns < 3 {
            errors.push(format!("columns must be >= 3 (got {})", self.columns));
        }
        if self.rows < 2 {
            errors.push(format!("rows must be >= 2 (got {})", self.rows));
        }
        if !(64.0..=96.0).contains(&self.icon_size) {
            errors.push(format!(
                "icon_size must be between 64.0 and 96.0 (got {})",
                self.icon_size
            ));
        }
        if !(0.4..=0.95).contains(&self.card_width) {
            errors.push(format!(
                "card_width must be between 0.40 and 0.95 (got {})",
                self.card_width
            ));
        }
        if !(0.4..=0.95).contains(&self.card_height) {
            errors.push(format!(
                "card_height must be between 0.40 and 0.95 (got {})",
                self.card_height
            ));
        }
        let clean_hex = self.accent_color.trim_start_matches('#');
        if (clean_hex.len() != 6 && clean_hex.len() != 8)
            || !clean_hex.chars().all(|c| c.is_ascii_hexdigit())
        {
            errors.push(format!(
                "accent_color must be #RRGGBB or #RRGGBBAA (got '{}')",
                self.accent_color
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let cfg = Config::default();
        assert_eq!(cfg.validate(), Ok(()));
    }

    #[test]
    fn test_config_toml_roundtrip() {
        let cfg = Config {
            columns: 8,
            rows: 4,
            icon_size: 84.0,
            centered: true,
            accent_color: "#00f5d4".to_string(),
            hotkey: "Ctrl+Space".to_string(),
            ..Config::default()
        };
        let raw = toml::to_string_pretty(&cfg).expect("serialize");
        let parsed: Config = toml::from_str(&raw).expect("deserialize");
        assert_eq!(cfg, parsed);
    }

    #[test]
    fn test_config_validation_catches_invalid_values() {
        let bad = Config {
            columns: 1,
            rows: 1,
            icon_size: 20.0,
            accent_color: "not-a-hex".to_string(),
            ..Config::default()
        };
        let errs = bad.validate().expect_err("expected validation errors");
        assert!(errs.len() >= 4);
    }
}
