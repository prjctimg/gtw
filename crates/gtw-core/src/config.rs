use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub network: NetworkConfig,
    pub cache: CacheConfig,
    pub render: RenderConfig,
    pub media: MediaConfig,
    pub effects: EffectsConfig,
    pub theme: ThemeConfig,
    pub auth: AuthConfig,
    pub editor: EditorConfig,
    pub keys: KeysConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub offline: bool,
    pub no_animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub user_agent: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub max_size_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderConfig {
    pub syntax_highlight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    pub use_keyring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub autosave: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct KeysConfig {}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            offline: false,
            no_animations: false,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            user_agent: "gtw/0.1.0 (https://github.com/prjctimg/gtw)".to_string(),
            timeout_secs: 30,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { max_size_mb: 512 }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            syntax_highlight: true,
        }
    }
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "Chadrula".to_string(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { use_keyring: true }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self { autosave: true }
    }
}
