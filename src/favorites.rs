use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Favorite {
    pub name: String,
    pub path: String,
}

pub struct FavoritesManager {
    favorites: Vec<Favorite>,
    config_path: PathBuf,
}

impl FavoritesManager {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let favorites = Self::load_favorites(&config_path);

        Self {
            favorites,
            config_path,
        }
    }

    fn get_config_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .ok()
                .map(|p| PathBuf::from(p).join("vibe-fm").join("favorites.json"))
                .unwrap_or_else(|| PathBuf::from("favorites.json"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            dirs::config_dir()
                .map(|p| p.join("vibe-fm").join("favorites.json"))
                .unwrap_or_else(|| PathBuf::from("favorites.json"))
        }
    }

    fn load_favorites(path: &PathBuf) -> Vec<Favorite> {
        if !path.exists() {
            let mut defaults = Vec::new();

            #[cfg(target_os = "windows")]
            {
                if let Some(home) = dirs::home_dir() {
                    defaults.push(Favorite {
                        name: "Home".to_string(),
                        path: home.to_string_lossy().to_string(),
                    });
                    defaults.push(Favorite {
                        name: "Desktop".to_string(),
                        path: dirs::desktop_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    });
                    defaults.push(Favorite {
                        name: "Documents".to_string(),
                        path: dirs::document_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    });
                    defaults.push(Favorite {
                        name: "Downloads".to_string(),
                        path: dirs::download_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    });
                }
            }

            return defaults;
        }

        match fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    pub fn save(&self) {
        if let Some(parent) = self.config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        if let Ok(content) = serde_json::to_string_pretty(&self.favorites) {
            let _ = fs::write(&self.config_path, content);
        }
    }

    pub fn get_favorites(&self) -> &Vec<Favorite> {
        &self.favorites
    }

    pub fn add_favorite(&mut self, name: String, path: String) {
        if !self.favorites.iter().any(|f| f.path == path) {
            self.favorites.push(Favorite { name, path });
            self.save();
        }
    }

    pub fn remove_favorite(&mut self, path: &str) {
        self.favorites.retain(|f| f.path != path);
        self.save();
    }
}

impl Default for FavoritesManager {
    fn default() -> Self {
        Self::new()
    }
}
