use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{self, Command, Output},
};

use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    hint::Hinter,
    history::{DefaultHistory, History},
    Context, DefaultEditor, Editor, Helper, Highlighter, Validator,
};

use super::{
    args::Cli,
    definitions::Definitions,
    manifest::{Dependencies, Manifest},
};

type FilenameEditor = Editor<FilenameParser, DefaultHistory>;

#[derive(Debug, Default)]
pub struct Interface {
    editor: Option<DefaultEditor>,
    filename_editor: Option<FilenameEditor>,
}

impl Interface {
    const PROMPT: &'static str = "> ";

    /// Lazy load the inner editor.
    fn editor(&mut self) -> anyhow::Result<&mut DefaultEditor> {
        if self.editor.is_none() {
            self.editor.replace(DefaultEditor::new()?);
        }

        self.editor
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg("unavailable editor"))
    }

    /// Lazy load the inner editor.
    fn filename_editor(&mut self) -> anyhow::Result<&mut FilenameEditor> {
        if self.filename_editor.is_none() {
            self.filename_editor.replace(FilenameEditor::new()?);
        }

        self.filename_editor
            .as_mut()
            .ok_or_else(|| anyhow::Error::msg("unavailable editor"))
    }

    fn read_line<T, U>(
        rl: &mut Editor<T, U>,
        args: &Cli,
        prompt: Option<&str>,
        mut initial_value: Option<&str>,
    ) -> String
    where
        T: Helper,
        U: History,
    {
        if args.defaults {
            if let Some(value) = initial_value {
                return value.to_string();
            }
        }

        if let Some(prompt) = prompt {
            println!("{}", prompt);
        }

        loop {
            let readline = match initial_value.take() {
                Some(initial) => rl.readline_with_initial(Self::PROMPT, (initial, "")),
                None => rl.readline(Self::PROMPT),
            };

            match readline {
                Ok(line) => return line,
                Err(ReadlineError::Interrupted) => {
                    process::exit(0);
                }
                Err(ReadlineError::Eof) => {
                    eprintln!("CTRL-D");
                    process::exit(0);
                }
                Err(err) => {
                    eprintln!("Input error: {}", err);
                }
            }
        }
    }

    fn expect_char(
        &mut self,
        args: &Cli,
        prompt: Option<&str>,
        initial_value: Option<char>,
        expected: char,
    ) -> anyhow::Result<()> {
        // TODO implement an actual read char function
        let rl = self.editor()?;
        let initial_value = initial_value.map(|c| c.to_string());
        let expected = expected.to_lowercase().to_string();
        let line = Self::read_line(rl, args, prompt, initial_value.as_ref().map(|s| s.as_str()))
            .to_lowercase();

        anyhow::ensure!(line == expected, "Expected {}, got {}", expected, line);
        Ok(())
    }

    fn read_path(
        &mut self,
        args: &Cli,
        prompt: Option<&str>,
        initial_value: Option<&str>,
    ) -> anyhow::Result<PathBuf> {
        let rl = self.filename_editor()?;
        let path = Self::read_line(rl, args, prompt, initial_value);

        Ok(PathBuf::from(path))
    }

    pub fn manifest(&mut self, args: &Cli) -> anyhow::Result<Manifest> {
        let dir_or_file = match args.path.as_ref().cloned() {
            Some(path) => path,
            None => {
                let cwd = env::current_dir()?.display().to_string();
                self.read_path(
                    args,
                    Some("Insert the path to your `Cargo.toml`"),
                    Some(&cwd),
                )?
            }
        };

        let path = if dir_or_file.is_dir() {
            dir_or_file.join("Cargo.toml")
        } else {
            dir_or_file
        };

        if !path.exists() {
            anyhow::bail!(
                "Failed to locate `Cargo.toml`; {} does not exist",
                path.display()
            );
        }

        if !path.is_file() {
            anyhow::bail!(
                "Failed to locate `Cargo.toml`; {} is not a file",
                path.display()
            );
        }

        let path = path.canonicalize()?;

        println!("Using manifest `{}`...", path.display());
        let mut manifest = Manifest::try_from(path.as_path())?;

        if let Dependencies::Unresolved {
            borsh,
            serde_json,
            sov_modules_api,
        } = &manifest.dependencies
        {
            let rl = self.editor()?;

            let prompt = Some("Enter the manifest dependency for the base library");
            let initial = format!("{{ path = \"{}\" }}", manifest.parent.display());
            let base = Self::read_line(rl, args, prompt, Some(&initial));

            let prompt = Some("Enter the manifest dependency for borsh");
            let borsh = Self::read_line(rl, args, prompt, borsh.as_ref().map(|s| s.as_str()));

            let prompt = Some("Enter the manifest dependency for serde_json");
            let serde_json =
                Self::read_line(rl, args, prompt, serde_json.as_ref().map(|s| s.as_str()));

            let prompt = Some("Enter the manifest dependency for sov-modules-api");
            let sov_modules_api = Self::read_line(
                rl,
                args,
                prompt,
                sov_modules_api.as_ref().map(|s| s.as_str()),
            );

            manifest.dependencies = Dependencies::Resolved {
                base,
                borsh,
                serde_json,
                sov_modules_api,
            };
        }

        println!("Reading project `{}`...", manifest.name);
        Ok(manifest)
    }

    pub fn target_dir(&mut self, args: &Cli, manifest: &Manifest) -> anyhow::Result<PathBuf> {
        let target = match args.target.as_ref().cloned() {
            Some(target) => target,
            None => {
                let name = format!("{}-snap", manifest.name);
                let target = manifest
                    .parent
                    .parent()
                    .unwrap_or(&manifest.parent)
                    .join(name)
                    .display()
                    .to_string();

                self.read_path(
                    args,
                    Some("Insert the target directory"),
                    Some(target.as_str()),
                )?
            }
        };

        if target.is_file() {
            anyhow::bail!(
                "The provided target `{}` is a file; use a directory",
                target.display()
            );
        }

        if target.exists() {
            if fs::remove_dir(&target).is_err() {
                if !args.force {
                    let prompt = format!(
                        "The provided target `{}` is not empty and will be erased; confirm? [y/n]",
                        target.display()
                    );

                    if self
                        .expect_char(args, Some(&prompt), Some('n'), 'y')
                        .is_err()
                    {
                        anyhow::bail!("Operation aborted");
                    }
                }

                fs::remove_dir_all(&target)?;
            }
        }

        Ok(target)
    }

    pub fn git_clone<P>(&mut self, args: &Cli, target: P) -> anyhow::Result<Output>
    where
        P: AsRef<Path>,
    {
        let origin = match args.origin.as_ref().cloned() {
            Some(origin) => origin,
            None => {
                let rl = self.editor()?;
                Self::read_line(
                    rl,
                    args,
                    Some("Insert the origin git repository of the snap template"),
                    Some("https://github.com/Sovereign-Labs/sov-snap"),
                )
            }
        };

        let branch = match args.branch.as_ref().cloned() {
            Some(branch) => branch,
            None => {
                let rl = self.editor()?;
                Self::read_line(
                    rl,
                    args,
                    Some("Insert the branch of the snap template"),
                    Some("v0.1.1"),
                )
            }
        };

        Command::new("git")
            .arg("clone")
            .arg("--quiet")
            .arg("--progress")
            .arg("-c")
            .arg("advice.detachedHead=false")
            .arg("--branch")
            .arg(branch)
            .arg("--single-branch")
            .arg("--depth")
            .arg("1")
            .arg(origin)
            .arg(target.as_ref())
            .output()
            .map_err(Into::into)
    }

    pub fn definitions(&mut self, args: &Cli, manifest: &Manifest) -> anyhow::Result<Definitions> {
        let context = match args.context.as_ref().cloned() {
            Some(context) => context,
            None => {
                let rl = self.editor()?;
                let initial = format!("{}::Context", manifest.name_replaced);
                Self::read_line(
                    rl,
                    args,
                    Some("Insert the path of the runtime context"),
                    Some(&initial),
                )
            }
        };

        let da_spec = match args.da_spec.as_ref().cloned() {
            Some(da_spec) => da_spec,
            None => {
                let rl = self.editor()?;
                let initial = format!("{}::DaSpec", manifest.name_replaced);
                Self::read_line(
                    rl,
                    args,
                    Some("Insert the path of the DA runtime spec"),
                    Some(&initial),
                )
            }
        };

        let runtime = match args.runtime.as_ref().cloned() {
            Some(runtime) => runtime,
            None => {
                let rl = self.editor()?;
                let initial = format!("{}::RuntimeCall<Context, DaSpec>", manifest.name_replaced);
                Self::read_line(
                    rl,
                    args,
                    Some("Insert the path of the runtime call"),
                    Some(&initial),
                )
            }
        };

        Ok(Definitions {
            context,
            da_spec,
            runtime,
        })
    }
}

#[derive(Helper, Validator, Highlighter)]
pub struct FilenameParser {
    filename_completer: FilenameCompleter,
}

impl Completer for FilenameParser {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        // TODO filename completion is not being called; rustyline bug?
        if line.is_empty() {
            // hints current working dir by default
            let cwd = env::current_dir()
                .expect("failed to read current working dir")
                .display()
                .to_string();

            let pair = Pair {
                display: cwd.clone(),
                replacement: cwd,
            };

            return Ok((0, vec![pair]));
        }

        self.filename_completer.complete_path(line, pos)
    }
}

impl Hinter for FilenameParser {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<Self::Hint> {
        self.complete(line, pos, ctx)
            .ok()
            .and_then(|(_, pairs)| pairs.first().cloned())
            .map(|pair| pair.replacement[pos..].to_string())
    }
}
