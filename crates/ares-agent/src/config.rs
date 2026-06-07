use ares_core::AresError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Agent configuration — loaded from ~/.ares/config.toml or project .ares/config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub project_path: String,
    pub project_id: Option<String>,
    pub ares_home: PathBuf,
    pub socket_path: PathBuf,
    pub log_level: String,
}

impl AgentConfig {
    pub fn load(project_path: &str) -> Result<Self, AresError> {
        let ares_home = dirs_home().join(".ares");
        std::fs::create_dir_all(&ares_home).map_err(AresError::Io)?;

        // Derive socket path from project path hash
        let path_hash = blake3_short(project_path);
        let socket_name = format!("ares-{}.sock", &path_hash[..8]);

        #[cfg(unix)]
        let socket_path = ares_home.join(&socket_name);

        #[cfg(windows)]
        let socket_path = PathBuf::from(format!(r"\\.\pipe\{}", socket_name.replace('.', "-")));

        Ok(Self {
            project_path: project_path.to_string(),
            project_id: None, // populated after init
            ares_home,
            socket_path,
            log_level: "info".into(),
        })
    }

    #[allow(dead_code)]
    pub fn db_path(&self, project_id: &str) -> PathBuf {
        self.ares_home
            .join("projects")
            .join(project_id)
            .join("ares.db")
    }

    #[allow(dead_code)]
    pub fn registry_path(&self) -> PathBuf {
        self.ares_home.join("registry.json")
    }
}

fn dirs_home() -> PathBuf {
    #[cfg(unix)]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    }
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .or_else(|_| {
                std::env::var("HOMEDRIVE")
                    .and_then(|d| std::env::var("HOMEPATH").map(|p| format!("{d}{p}")))
            })
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\default"))
    }
}

fn blake3_short(input: &str) -> String {
    blake3::hash(input.as_bytes()).to_hex().to_string()
}
