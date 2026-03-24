//! Domain types shared across all nexus crates.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Category for a discovered file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Config,
    Document,
    Image,
    Video,
    Audio,
    Code,
    Archive,
    Cache,
    Project,
    Download,
    Other,
}

impl FileCategory {
    /// Categorize a file by its path and extension.
    pub fn from_path(path: &std::path::Path) -> Self {
        // Check path components first
        let path_str = path.to_string_lossy();
        if path_str.contains("/.config/") || path_str.contains("/dotfiles/") {
            return Self::Config;
        }
        if path_str.contains("/Library/Caches/")
            || path_str.contains("/.cache/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/target/")
        {
            return Self::Cache;
        }
        if path_str.contains("/Downloads/") {
            return Self::Download;
        }

        // Then check extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // Code
            "rs" | "py" | "js" | "ts" | "go" | "java" | "c" | "cpp" | "h" | "svelte" | "vue"
            | "rb" | "swift" | "kt" | "sh" | "fish" | "zsh" | "bash" | "lua" | "vim" | "el" => {
                Self::Code
            }
            // Config
            "toml" | "yaml" | "yml" | "json" | "ini" | "conf" | "cfg" => Self::Config,
            // Documents
            "md" | "txt" | "pdf" | "doc" | "docx" | "rtf" | "tex" | "org" => Self::Document,
            // Images
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" | "heic" => Self::Image,
            // Video
            "mp4" | "mkv" | "avi" | "mov" | "webm" => Self::Video,
            // Audio
            "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac" => Self::Audio,
            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "dmg" => Self::Archive,
            // Project markers
            "lock" | "sum" => Self::Project,
            _ => Self::Other,
        }
    }

    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Document => "document",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::Code => "code",
            Self::Archive => "archive",
            Self::Cache => "cache",
            Self::Project => "project",
            Self::Download => "download",
            Self::Other => "other",
        }
    }

    /// Parse from database string.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "config" => Self::Config,
            "document" => Self::Document,
            "image" => Self::Image,
            "video" => Self::Video,
            "audio" => Self::Audio,
            "code" => Self::Code,
            "archive" => Self::Archive,
            "cache" => Self::Cache,
            "project" => Self::Project,
            "download" => Self::Download,
            _ => Self::Other,
        }
    }
}

/// A file discovered during scanning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub id: Option<i64>,
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
    pub modified_at: i64,
    pub created_at: Option<i64>,
    pub is_dir: bool,
    pub depth: u32,
    pub parent_path: Option<String>,
    pub category: FileCategory,
}

/// A config tool discovered in ~/.config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTool {
    pub id: Option<i64>,
    pub name: String,
    pub config_dir: PathBuf,
    pub description: String,
    pub language: String,
    pub total_size: u64,
    pub file_count: u32,
    pub last_modified: i64,
}

/// A config file belonging to a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    pub id: Option<i64>,
    pub tool_id: i64,
    pub tool_name: String,
    pub path: PathBuf,
    pub content_hash: String,
    pub size: u64,
    pub modified_at: i64,
    pub language: Option<String>,
}

/// A snapshot of config state for backup/restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub id: i64,
    pub tool_id: Option<i64>,
    pub tool_name: Option<String>,
    pub label: Option<String>,
    pub created_at: i64,
    pub file_count: u32,
}

/// A file change detected by the watcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub id: Option<i64>,
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub detected_at: i64,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
}

/// Type of filesystem change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

impl ChangeType {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
        }
    }
}

/// Search query for FTS5.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub category: Option<FileCategory>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub limit: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            category: None,
            min_size: None,
            max_size: None,
            limit: 100,
        }
    }
}

/// Search result from FTS5.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub category: FileCategory,
    pub modified_at: i64,
    pub rank: f64,
}

/// Progress update during scanning.
#[derive(Debug, Clone)]
pub struct Progress {
    pub files_scanned: u64,
    pub total_size: u64,
    pub current_path: String,
}

/// Summary statistics for the home directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeStats {
    pub total_files: u64,
    pub total_dirs: u64,
    pub total_size: u64,
    pub by_category: Vec<CategoryStats>,
    pub last_scan: Option<i64>,
}

/// Stats for a single category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: FileCategory,
    pub file_count: u64,
    pub total_size: u64,
}

/// Known tool definition for config registry.
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub config_path: &'static str,
    pub file_patterns: &'static [&'static str],
    pub language: &'static str,
    pub description: &'static str,
}

/// All known config tools.
pub const KNOWN_TOOLS: &[ToolDefinition] = &[
    ToolDefinition {
        name: "alacritty",
        config_path: ".config/alacritty",
        file_patterns: &["alacritty.toml", "alacritty.yml"],
        language: "toml",
        description: "GPU-accelerated terminal",
    },
    ToolDefinition {
        name: "atuin",
        config_path: ".config/atuin",
        file_patterns: &["config.toml"],
        language: "toml",
        description: "Shell history sync",
    },
    ToolDefinition {
        name: "btop",
        config_path: ".config/btop",
        file_patterns: &["btop.conf", "*.conf"],
        language: "conf",
        description: "Resource monitor",
    },
    ToolDefinition {
        name: "fish",
        config_path: ".config/fish",
        file_patterns: &["config.fish", "*.fish"],
        language: "fish",
        description: "Fish shell",
    },
    ToolDefinition {
        name: "flutter",
        config_path: ".config/flutter",
        file_patterns: &["settings", "*.json"],
        language: "json",
        description: "Flutter SDK",
    },
    ToolDefinition {
        name: "gcloud",
        config_path: ".config/gcloud",
        file_patterns: &["*.json", "properties"],
        language: "json",
        description: "Google Cloud SDK",
    },
    ToolDefinition {
        name: "gh",
        config_path: ".config/gh",
        file_patterns: &["config.yml", "hosts.yml"],
        language: "yaml",
        description: "GitHub CLI",
    },
    ToolDefinition {
        name: "git",
        config_path: ".config/git",
        file_patterns: &["config", "ignore", "attributes"],
        language: "ini",
        description: "Git version control",
    },
    ToolDefinition {
        name: "gitui",
        config_path: ".config/gitui",
        file_patterns: &["key_bindings.ron", "theme.ron"],
        language: "rust",
        description: "Git terminal UI",
    },
    ToolDefinition {
        name: "helix",
        config_path: ".config/helix",
        file_patterns: &["config.toml", "languages.toml", "themes/*.toml"],
        language: "toml",
        description: "Post-modern text editor",
    },
    ToolDefinition {
        name: "katana",
        config_path: ".config/katana",
        file_patterns: &["*.toml", "*.json"],
        language: "toml",
        description: "Katana",
    },
    ToolDefinition {
        name: "kitty",
        config_path: ".config/kitty",
        file_patterns: &["kitty.conf", "*.conf"],
        language: "conf",
        description: "GPU-based terminal emulator",
    },
    ToolDefinition {
        name: "lazygit",
        config_path: ".config/lazygit",
        file_patterns: &["config.yml", "state.yml"],
        language: "yaml",
        description: "Terminal UI for git",
    },
    ToolDefinition {
        name: "mise",
        config_path: ".config/mise",
        file_patterns: &["config.toml", "settings.toml"],
        language: "toml",
        description: "Dev tool version manager",
    },
    ToolDefinition {
        name: "nushell",
        config_path: ".config/nushell",
        file_patterns: &["config.nu", "env.nu", "*.nu"],
        language: "nu",
        description: "Nushell",
    },
    ToolDefinition {
        name: "nvim",
        config_path: ".config/nvim",
        file_patterns: &["init.lua", "*.lua", "*.vim"],
        language: "lua",
        description: "Neovim editor",
    },
    ToolDefinition {
        name: "raycast",
        config_path: ".config/raycast",
        file_patterns: &["*.json"],
        language: "json",
        description: "Raycast launcher",
    },
    ToolDefinition {
        name: "sniffnet",
        config_path: ".config/sniffnet",
        file_patterns: &["*.toml"],
        language: "toml",
        description: "Network monitor",
    },
    ToolDefinition {
        name: "starship",
        config_path: ".config/starship.toml",
        file_patterns: &["starship.toml"],
        language: "toml",
        description: "Shell prompt",
    },
    ToolDefinition {
        name: "stripe",
        config_path: ".config/stripe",
        file_patterns: &["config.toml"],
        language: "toml",
        description: "Stripe CLI",
    },
    ToolDefinition {
        name: "tmux",
        config_path: ".config/tmux",
        file_patterns: &["tmux.conf", "*.conf"],
        language: "conf",
        description: "Tmux terminal multiplexer",
    },
    ToolDefinition {
        name: "uv",
        config_path: ".config/uv",
        file_patterns: &["uv.toml"],
        language: "toml",
        description: "Python package manager",
    },
    ToolDefinition {
        name: "wezterm",
        config_path: ".config/wezterm",
        file_patterns: &["wezterm.lua", "*.lua"],
        language: "lua",
        description: "WezTerm terminal",
    },
    ToolDefinition {
        name: "zellij",
        config_path: ".config/zellij",
        file_patterns: &["config.kdl", "*.kdl"],
        language: "kdl",
        description: "Terminal multiplexer",
    },
    ToolDefinition {
        name: "zsh",
        config_path: ".config/zsh",
        file_patterns: &[".zshrc", ".zprofile", ".zshenv", "*.zsh"],
        language: "zsh",
        description: "Zsh shell",
    },
];
