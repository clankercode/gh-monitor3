use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github_token: Option<String>,
    pub repos: Vec<RepoConfig>,
    pub orgs: Vec<String>,
    pub poll_interval_secs: u64,
    pub window: WindowConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub owner: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: u32,
    pub height: u32,
    pub opacity: f32,
    pub hover_opacity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background_color: [f32; 4],
    pub text_color: [f32; 4],
    pub badge_colors: BadgeColors,
    pub font_size: f32,
    pub font_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeColors {
    pub pr: [f32; 4],
    pub issue: [f32; 4],
    pub push: [f32; 4],
    pub release: [f32; 4],
    pub fork: [f32; 4],
    pub create: [f32; 4],
    pub other: [f32; 4],
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github_token: std::env::var("GITHUB_TOKEN")
                .ok()
                .or_else(crate::github::client::GithubClient::get_gh_cli_token),
            repos: Vec::new(),
            orgs: Vec::new(),
            poll_interval_secs: 600,
            window: WindowConfig::default(),
            theme: ThemeConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 320,
            height: 480,
            opacity: 0.15,
            hover_opacity: 0.95,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: [0.05, 0.05, 0.08, 1.0],
            text_color: [0.9, 0.9, 0.92, 1.0],
            badge_colors: BadgeColors::default(),
            font_size: 13.0,
            font_path: None,
        }
    }
}

impl Default for BadgeColors {
    fn default() -> Self {
        Self {
            pr: [0.2, 0.8, 0.4, 1.0],
            issue: [0.3, 0.5, 0.9, 1.0],
            push: [0.5, 0.5, 0.5, 1.0],
            release: [0.7, 0.3, 0.9, 1.0],
            fork: [0.9, 0.6, 0.2, 1.0],
            create: [0.95, 0.8, 0.2, 1.0],
            other: [0.4, 0.4, 0.4, 1.0],
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> crate::error::Result<Self> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => {
                let mut path = dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("gh-monitor3");
                path.push("config.toml");
                path
            }
        };

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    pub fn save(&self, path: Option<&str>) -> crate::error::Result<()> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => {
                let mut path = dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("gh-monitor3");
                std::fs::create_dir_all(&path)?;
                path.push("config.toml");
                path
            }
        };

        let mut save_config = self.clone();
        if save_config.github_token.is_some() {
            save_config.github_token = None;
        }

        let contents = toml::to_string_pretty(&save_config)
            .map_err(|e| crate::error::AppError::Config(e.to_string()))?;
        std::fs::write(&config_path, contents)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn save_token(&self, path: Option<&str>) -> crate::error::Result<()> {
        if let Some(ref token) = self.github_token {
            let token_path = match path {
                Some(p) => std::path::PathBuf::from(p).with_extension("token"),
                None => {
                    let mut path = dirs::config_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("gh-monitor3");
                    std::fs::create_dir_all(&path)?;
                    path.push(".token");
                    path
                }
            };
            std::fs::write(&token_path, token)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.poll_interval_secs, 600);
        assert_eq!(config.window.width, 320);
        assert_eq!(config.window.height, 480);
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.poll_interval_secs, parsed.poll_interval_secs);
    }

    #[test]
    fn test_default_window_config() {
        let w = WindowConfig::default();
        assert!(w.x.is_none());
        assert!(w.y.is_none());
        assert_eq!(w.width, 320);
        assert_eq!(w.height, 480);
        assert_eq!(w.opacity, 0.15);
        assert_eq!(w.hover_opacity, 0.95);
    }

    #[test]
    fn test_default_theme_config() {
        let t = ThemeConfig::default();
        assert_eq!(t.background_color, [0.05, 0.05, 0.08, 1.0]);
        assert_eq!(t.text_color, [0.9, 0.9, 0.92, 1.0]);
        assert_eq!(t.font_size, 13.0);
        assert!(t.font_path.is_none());
    }

    #[test]
    fn test_default_badge_colors() {
        let b = BadgeColors::default();
        assert_eq!(b.pr, [0.2, 0.8, 0.4, 1.0]);
        assert_eq!(b.issue, [0.3, 0.5, 0.9, 1.0]);
        assert_eq!(b.push, [0.5, 0.5, 0.5, 1.0]);
        assert_eq!(b.release, [0.7, 0.3, 0.9, 1.0]);
        assert_eq!(b.fork, [0.9, 0.6, 0.2, 1.0]);
        assert_eq!(b.create, [0.95, 0.8, 0.2, 1.0]);
        assert_eq!(b.other, [0.4, 0.4, 0.4, 1.0]);
    }

    #[test]
    fn test_default_config_has_empty_repos() {
        let config = Config::default();
        assert!(config.repos.is_empty());
        assert!(config.orgs.is_empty());
    }

    #[test]
    fn test_default_config_token_from_env() {
        let config = Config::default();
        let expected = std::env::var("GITHUB_TOKEN")
            .ok()
            .or_else(crate::github::client::GithubClient::get_gh_cli_token);
        assert_eq!(config.github_token, expected);
    }

    #[test]
    fn test_load_from_file() {
        let dir = std::env::temp_dir().join("gh-monitor3-test-load");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let toml_content = r#"
github_token = "test-token-123"
poll_interval_secs = 30

repos = []
orgs = []

[window]
width = 640
height = 960
opacity = 0.5
hover_opacity = 1.0

[theme]
background_color = [0.0, 0.0, 0.0, 1.0]
text_color = [1.0, 1.0, 1.0, 1.0]
font_size = 16.0

[theme.badge_colors]
pr = [1.0, 0.0, 0.0, 1.0]
issue = [0.0, 1.0, 0.0, 1.0]
push = [0.0, 0.0, 1.0, 1.0]
release = [1.0, 1.0, 0.0, 1.0]
fork = [0.0, 1.0, 1.0, 1.0]
create = [1.0, 0.0, 1.0, 1.0]
other = [0.5, 0.5, 0.5, 1.0]
"#;
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load(Some(path.to_str().unwrap())).unwrap();
        assert_eq!(config.github_token, Some("test-token-123".to_string()));
        assert_eq!(config.poll_interval_secs, 30);
        assert_eq!(config.window.width, 640);
        assert_eq!(config.window.height, 960);
        assert_eq!(config.window.opacity, 0.5);
        assert_eq!(config.window.hover_opacity, 1.0);
        assert_eq!(config.theme.font_size, 16.0);
        assert_eq!(config.theme.background_color, [0.0, 0.0, 0.0, 1.0]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = std::env::temp_dir().join("gh-monitor3-test-roundtrip");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let mut config = Config::default();
        config.github_token = Some("roundtrip-token".to_string());
        config.poll_interval_secs = 120;
        config.repos = vec![
            RepoConfig {
                owner: "octocat".to_string(),
                name: "hello-world".to_string(),
            },
            RepoConfig {
                owner: "rust-lang".to_string(),
                name: "rust".to_string(),
            },
        ];
        config.orgs = vec!["my-org".to_string()];
        config.window.width = 800;
        config.window.height = 600;
        config.window.x = Some(100);
        config.window.y = Some(200);
        config.theme.font_size = 18.0;
        config.theme.font_path = Some("/usr/share/fonts/test.ttf".to_string());

        config.save(Some(path.to_str().unwrap())).unwrap();
        let loaded = Config::load(Some(path.to_str().unwrap())).unwrap();

        assert!(loaded.github_token.is_none());
        assert_eq!(loaded.poll_interval_secs, 120);
        assert_eq!(loaded.repos.len(), 2);
        assert_eq!(loaded.repos[0].owner, "octocat");
        assert_eq!(loaded.repos[0].name, "hello-world");
        assert_eq!(loaded.repos[1].owner, "rust-lang");
        assert_eq!(loaded.repos[1].name, "rust");
        assert_eq!(loaded.orgs, vec!["my-org".to_string()]);
        assert_eq!(loaded.window.width, 800);
        assert_eq!(loaded.window.height, 600);
        assert_eq!(loaded.window.x, Some(100));
        assert_eq!(loaded.window.y, Some(200));
        assert_eq!(loaded.theme.font_size, 18.0);
        assert_eq!(
            loaded.theme.font_path,
            Some("/usr/share/fonts/test.ttf".to_string())
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_missing_config_file_returns_default() {
        let config = Config::load(Some("/tmp/nonexistent-gh-monitor3-config-xyz.toml")).unwrap();
        assert_eq!(config.poll_interval_secs, 600);
        assert_eq!(config.window.width, 320);
        assert!(config.repos.is_empty());
    }

    #[test]
    fn test_save_creates_parent_directories() {
        let dir = std::env::temp_dir()
            .join("gh-monitor3-test-nested")
            .join("sub")
            .join("dir");
        let _ = std::fs::remove_dir_all(std::env::temp_dir().join("gh-monitor3-test-nested"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        let config = Config::default();
        config.save(Some(path.to_str().unwrap())).unwrap();

        assert!(path.exists());
        let loaded = Config::load(Some(path.to_str().unwrap())).unwrap();
        assert_eq!(loaded.poll_interval_secs, 600);

        let _ = std::fs::remove_dir_all(std::env::temp_dir().join("gh-monitor3-test-nested"));
    }

    #[test]
    fn test_load_with_repos_and_orgs() {
        let dir = std::env::temp_dir().join("gh-monitor3-test-repos-orgs");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let toml_content = r#"
github_token = "tok"
poll_interval_secs = 45

repos = [
    { owner = "a", name = "b" },
    { owner = "c", name = "d" },
]

orgs = ["org1", "org2"]

[window]
width = 100
height = 200
opacity = 0.3
hover_opacity = 0.8

[theme]
background_color = [0.1, 0.2, 0.3, 1.0]
text_color = [0.9, 0.8, 0.7, 1.0]
font_size = 14.0

[theme.badge_colors]
pr = [1.0, 0.0, 0.0, 1.0]
issue = [0.0, 1.0, 0.0, 1.0]
push = [0.0, 0.0, 1.0, 1.0]
release = [1.0, 1.0, 0.0, 1.0]
fork = [0.0, 1.0, 1.0, 1.0]
create = [1.0, 0.0, 1.0, 1.0]
other = [0.5, 0.5, 0.5, 1.0]
"#;
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::load(Some(path.to_str().unwrap())).unwrap();
        assert_eq!(config.repos.len(), 2);
        assert_eq!(config.repos[0].owner, "a");
        assert_eq!(config.repos[0].name, "b");
        assert_eq!(config.repos[1].owner, "c");
        assert_eq!(config.repos[1].name, "d");
        assert_eq!(config.orgs, vec!["org1", "org2"]);
        assert_eq!(config.poll_interval_secs, 45);
        assert_eq!(config.window.width, 100);
        assert_eq!(config.window.height, 200);
        assert_eq!(config.window.opacity, 0.3);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_serialize_produces_valid_toml() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("poll_interval_secs"));
        assert!(toml_str.contains("[window]"));
        assert!(toml_str.contains("[theme]"));
        assert!(toml_str.contains("[theme.badge_colors]"));
    }

    #[test]
    fn test_config_with_optional_fields_none() {
        let dir = std::env::temp_dir().join("gh-monitor3-test-optional");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let config = Config {
            github_token: None,
            repos: Vec::new(),
            orgs: Vec::new(),
            poll_interval_secs: 10,
            window: WindowConfig {
                x: None,
                y: None,
                width: 100,
                height: 100,
                opacity: 0.1,
                hover_opacity: 0.9,
            },
            theme: ThemeConfig {
                background_color: [0.0, 0.0, 0.0, 1.0],
                text_color: [1.0, 1.0, 1.0, 1.0],
                badge_colors: BadgeColors::default(),
                font_size: 12.0,
                font_path: None,
            },
        };

        config.save(Some(path.to_str().unwrap())).unwrap();
        let loaded = Config::load(Some(path.to_str().unwrap())).unwrap();
        assert!(loaded.github_token.is_none());
        assert!(loaded.theme.font_path.is_none());
        assert!(loaded.window.x.is_none());
        assert!(loaded.window.y.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
