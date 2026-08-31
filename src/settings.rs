use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    Auto,
    Light,
    Dark,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub theme: ThemeMode,
}

impl Settings {
    pub fn load() -> (Self, Option<String>) {
        let path = config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(settings) => (settings, None),
                Err(error) => (Self::default(), Some(format!("Invalid settings: {error}"))),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => (Self::default(), None),
            Err(error) => (
                Self::default(),
                Some(format!("Could not read settings: {error}")),
            ),
        }
    }

    pub fn save(&self) -> io::Result<()> {
        save_to(&config_path(), self)
    }
}

pub fn config_path() -> PathBuf {
    if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(path)
            .join("keyboard-voice")
            .join("config.toml");
    }
    directories::BaseDirs::new()
        .map(|base| base.config_dir().join("keyboard-voice").join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"))
}

fn save_to(path: &Path, settings: &Settings) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".config.toml.{}.tmp", std::process::id()));
    let contents = toml::to_string_pretty(settings).map_err(io::Error::other)?;
    fs::write(&temporary, contents)?;
    fs::rename(temporary, path)
}

pub fn system_prefers_dark() -> bool {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .to_ascii_lowercase()
                    .contains("dark")
            })
            .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            let value = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
            if value.contains("prefer-dark") || value.contains("dark") {
                return true;
            }
        }
        return env::var("GTK_THEME")
            .map(|value| value.to_ascii_lowercase().contains("dark"))
            .unwrap_or(false);
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_auto() {
        assert_eq!(Settings::default().theme, ThemeMode::Auto);
    }

    #[test]
    fn serializes_lowercase_theme() {
        let text = toml::to_string(&Settings {
            theme: ThemeMode::Dark,
        })
        .unwrap();
        assert!(text.contains("theme = \"dark\""));
    }
}
