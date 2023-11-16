use std::{
    fs,
    path::{Path, PathBuf},
};

use toml::Table;

pub struct Manifest {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub name: String,
    pub name_replaced: String,
    pub dependencies: Dependencies,
}

impl TryFrom<&Path> for Manifest {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::Error::msg("No parent path for the manifest dir"))?
            .to_path_buf();

        let manifest = fs::read_to_string(path)?;
        let manifest: Table = toml::from_str(&manifest)?;

        let name = manifest["package"]["name"]
            .as_str()
            .ok_or_else(|| anyhow::Error::msg("Invalid `package.name`"))?
            .to_string();

        let dependencies = Dependencies::from(&manifest);
        let name_replaced = name.replace("-", "_");

        Ok(Self {
            parent,
            path: path.to_path_buf(),
            name,
            name_replaced,
            dependencies,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependencies {
    Unresolved {
        borsh: Option<String>,
        serde_json: Option<String>,
        sov_modules_api: Option<String>,
    },
    Resolved {
        base: String,
        borsh: String,
        serde_json: String,
        sov_modules_api: String,
    },
}

impl From<&Table> for Dependencies {
    fn from(manifest: &Table) -> Self {
        let dependencies = &manifest["dependencies"];
        let borsh = dependencies.get("borsh").map(|d| d.to_string());
        let serde_json = dependencies.get("serde_json").map(|d| d.to_string());
        let sov_modules_api = dependencies.get("sov-modules-api").map(|d| d.to_string());

        Self::Unresolved {
            borsh,
            serde_json,
            sov_modules_api,
        }
    }
}
