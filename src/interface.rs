use std::{path::PathBuf, process, str};

use console::{Key, Term};
use rustyline::{
    completion::FilenameCompleter, error::ReadlineError, hint::Hinter, Context, Editor,
};
use rustyline_derive::{Completer, Helper, Highlighter, Validator};

use super::args::InterfaceArgs;

#[derive(Completer, Helper, Validator, Highlighter)]
pub struct LineParser {
    filename_completer: FilenameCompleter,
}

impl Hinter for LineParser {
    type Hint = String;

    fn hint(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        self.filename_completer
            .complete_path(line, line.len())
            .ok()
            .and_then(|(_, pairs)| {
                pairs
                    .first()
                    .map(|pair| pair.replacement[line.len()..].to_string())
            })
    }
}

pub struct Interface {
    args: InterfaceArgs,
    rl: Editor<()>,
    rl_path: Editor<LineParser>,
}

impl Interface {
    const PROMPT: &'static str = "> ";

    pub fn info<M: AsRef<str>>(&self, message: M) {
        if self.args.quiet < 1 {
            println!("{}", message.as_ref());
        }
    }

    pub fn error<M: AsRef<str>>(&self, message: M) {
        if self.args.quiet < 2 {
            eprintln!("{}", message.as_ref());
        }
    }

    pub fn prompt<M: AsRef<str>>(&self, message: M) {
        if self.args.quiet < 3 {
            println!("{}", message.as_ref());
        }
    }

    pub fn bail<M: AsRef<str>>(&self, message: M) {
        self.error(message);
        process::exit(1);
    }

    pub fn read_line(&mut self, mut initial_value: Option<&str>) -> String {
        if self.args.defaults {
            if let Some(initial) = initial_value {
                return initial.to_string();
            }
        }

        loop {
            let rl = match initial_value.take() {
                Some(initial) => self.rl.readline_with_initial(Self::PROMPT, (initial, "")),
                None => self.rl.readline(Self::PROMPT),
            };

            match rl {
                Ok(line) => return line,
                Err(ReadlineError::Interrupted) => {
                    self.bail("CTRL-C");
                }
                Err(ReadlineError::Eof) => {
                    self.bail("CTRL-D");
                }
                Err(err) => {
                    self.error(format!("error reading line: {}", err));
                }
            }
        }
    }

    pub fn read_path(&mut self, mut initial_value: Option<&str>) -> PathBuf {
        if self.args.defaults {
            if let Some(initial) = initial_value {
                return PathBuf::from(initial);
            }
        }

        loop {
            let rl = match initial_value.take() {
                Some(initial) => self
                    .rl_path
                    .readline_with_initial(Self::PROMPT, (initial, "")),
                None => self.rl_path.readline(Self::PROMPT),
            };

            match rl {
                Ok(line) => return PathBuf::from(line),
                Err(ReadlineError::Interrupted) => {
                    //self.warn("CTRL-C");
                }
                Err(ReadlineError::Eof) => {
                    self.bail("CTRL-D");
                }
                Err(err) => {
                    self.error(format!("error reading path: {}", err));
                }
            }
        }
    }

    pub fn read_confirmation(&self) {
        if self.args.force {
            return;
        }

        match Term::stdout().read_key() {
            Ok(Key::Char('y')) | Ok(Key::Char('Y')) => (),
            Ok(_) => self.bail("Operation aborted"),
            Err(e) => self.bail(format!("error reading char: {}", e)),
        }
    }

    pub fn path_or_read(&mut self, initial_value: Option<&str>, path: Option<PathBuf>) -> PathBuf {
        if let Some(p) = path {
            return p;
        }

        self.read_path(initial_value)
    }

    pub fn line_or_read(&mut self, initial_value: Option<&str>, path: Option<String>) -> String {
        if let Some(p) = path {
            return p;
        }

        self.read_line(initial_value)
    }
}

impl TryFrom<InterfaceArgs> for Interface {
    type Error = anyhow::Error;

    fn try_from(args: InterfaceArgs) -> anyhow::Result<Self> {
        let rl = Editor::new()?;
        let rl_path = Editor::new()?;

        Ok(Self { args, rl, rl_path })
    }
}
