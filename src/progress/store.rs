use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::progress::model::ProgressState;

pub struct ProgressStore {
    path: PathBuf,
}

impl ProgressStore {
    pub fn default() -> Result<Self> {
        let mut path = dirs::home_dir().context("cannot resolve home directory")?;
        path.push(".cka-coach");
        fs::create_dir_all(&path)?;
        path.push("progress.json");
        Ok(Self { path })
    }

    pub fn load(&self) -> Result<ProgressState> {
        if !Path::new(&self.path).exists() {
            return Ok(ProgressState::default());
        }

        let raw = fs::read_to_string(&self.path)
            .with_context(|| format!("failed reading {}", self.path.display()))?;
        let parsed = serde_json::from_str::<ProgressState>(&raw)
            .with_context(|| format!("failed parsing {}", self.path.display()))?;
        Ok(parsed)
    }

    pub fn save(&self, progress: &ProgressState) -> Result<()> {
        let payload = serde_json::to_string_pretty(progress)?;
        fs::write(&self.path, payload)
            .with_context(|| format!("failed writing {}", self.path.display()))?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
