use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;
use toml::Table;

use super::interface::Interface;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependency {
    pub name: String,
    pub formatted: String,
}

impl Dependency {
    pub fn new<S>(dep: S) -> Self
    where
        S: AsRef<str>,
    {
        let str = dep.as_ref();
        let name = str.to_string();
        let formatted = str.replace("-", "_");

        Self { name, formatted }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub project: Dependency,
    pub dependencies: HashMap<Dependency, String>,
}

impl Manifest {
    fn normalize<P>(mut deps: HashMap<Dependency, String>, path: P) -> HashMap<Dependency, String>
    where
        P: AsRef<Path>,
    {
        let path = match path.as_ref().parent() {
            Some(p) => p,
            None => return deps,
        };

        for (_, v) in deps.iter_mut() {
            if let Ok(t) = toml::from_str::<Table>(&format!("dep = {}", v)) {
                if let Some(p) = t.get("dep") {
                    if let Some(p) = p.as_table() {
                        if let Some(p) = p.get("path") {
                            if let Some(p) = p.as_str() {
                                if let Ok(p) = path.join(p).canonicalize() {
                                    *v = format!("{{ path = \"{}\" }}", p.display()).into();
                                }
                            }
                        }
                    }
                }
            }
        }

        deps
    }

    fn resolve_workspace<P>(
        workspace: P,
        interface: &mut Interface,
    ) -> Option<HashMap<Dependency, String>>
    where
        P: AsRef<Path>,
    {
        let mut workspace = workspace.as_ref().parent()?.parent()?;

        loop {
            let path = workspace.join("Cargo.toml");
            if path.exists() {
                let manifest = fs::read_to_string(&path).ok()?;
                let manifest: Table = toml::from_str(&manifest).ok()?;
                let deps = manifest
                    .get("workspace")?
                    .as_table()?
                    .get("dependencies")?
                    .as_table()?
                    .into_iter()
                    .map(|(dep, v)| (Dependency::new(dep), v.to_string()))
                    .collect::<HashMap<_, _>>();

                interface.info(format!(
                    "Using workspace dependencies from `{}`",
                    workspace.display()
                ));

                return Some(Self::normalize(deps, path));
            }

            workspace = workspace.parent()?;
        }
    }

    pub fn read<P>(path: P, interface: &mut Interface) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let manifest = fs::read_to_string(path)?;
        let manifest: Table = toml::from_str(&manifest)?;

        let project = manifest
            .get("package")
            .with_context(|| format!("Could not find `package` on `{}`", path.display()))?
            .get("name")
            .with_context(|| format!("Could not find `name` on `{}`", path.display()))?
            .as_str()
            .with_context(|| format!("Invalid `name` on `{}`", path.display()))?;
        let project = Dependency::new(project);

        let workspace = Self::resolve_workspace(path, interface);
        let manifest = manifest
            .get("dependencies")
            .with_context(|| format!("Could not find `dependencies` in `{}`", path.display()))?
            .as_table()
            .with_context(|| format!("Invalid `dependencies` in `{}`", path.display()))?;

        let mut dependencies = HashMap::new();

        for (dep, v) in manifest {
            let dep = Dependency::new(dep);
            let mut val = v.to_string();

            if let Some(t) = v.as_table() {
                if let Some(t) = t.get("workspace") {
                    if let Some(b) = t.as_bool() {
                        if b {
                            if let Some(w) = &workspace {
                                if let Some(d) = w.get(&dep) {
                                    val = d.to_string();
                                }
                            }
                        }
                    }
                }
            }

            dependencies.insert(dep, val);
        }

        let dependencies = Self::normalize(dependencies, path);

        interface.info(format!("Parsed dependencies from `{}`", path.display()));

        Ok(Self {
            project,
            dependencies,
        })
    }
}
