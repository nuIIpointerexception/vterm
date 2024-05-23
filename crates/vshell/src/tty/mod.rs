use std::{collections::HashMap, env, io, path::PathBuf, sync::Arc};

use polling::{Event, PollMode, Poller};

#[cfg(not(windows))]
mod unix;

#[cfg(not(windows))]
pub use self::unix::*;

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use self::windows::*;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Options {
    pub shell: Option<Shell>,

    pub working_directory: Option<PathBuf>,

    pub hold: bool,

    pub env: HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Shell {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

impl Shell {
    pub fn new(program: String, args: Vec<String>) -> Self {
        Self { program, args }
    }
}

pub trait EventedReadWrite {
    type Reader: io::Read;
    type Writer: io::Write;

    unsafe fn register(
        &mut self,
        _: &Arc<Poller>,
        _: Event,
        _: PollMode,
    ) -> io::Result<()>;
    fn reregister(
        &mut self,
        _: &Arc<Poller>,
        _: Event,
        _: PollMode,
    ) -> io::Result<()>;
    fn deregister(&mut self, _: &Arc<Poller>) -> io::Result<()>;

    fn reader(&mut self) -> &mut Self::Reader;
    fn writer(&mut self) -> &mut Self::Writer;
}

#[derive(Debug, PartialEq, Eq)]
pub enum ChildEvent {
    Exited(Option<i32>),
}

pub trait EventedPty: EventedReadWrite {
    fn next_child_event(&mut self) -> Option<ChildEvent>;
}

pub fn setup_env() {
    let terminfo = if terminfo_exists("alacritty") {
        "alacritty"
    } else {
        "xterm-256color"
    };
    env::set_var("TERM", terminfo);

    env::set_var("COLORTERM", "truecolor");

    env::remove_var("DESKTOP_STARTUP_ID");
    env::remove_var("XDG_ACTIVATION_TOKEN");
}

fn terminfo_exists(terminfo: &str) -> bool {
    let first = terminfo.get(..1).unwrap_or_default();
    let first_hex =
        format!("{:x}", first.chars().next().unwrap_or_default() as usize);

    macro_rules! check_path {
        ($path:expr) => {
            if $path.join(first).join(terminfo).exists()
                || $path.join(&first_hex).join(terminfo).exists()
            {
                return true;
            }
        };
    }

    if let Some(dir) = env::var_os("TERMINFO") {
        check_path!(PathBuf::from(&dir));
    } else if let Some(home) = home::home_dir() {
        check_path!(home.join(".terminfo"));
    }

    if let Ok(dirs) = env::var("TERMINFO_DIRS") {
        for dir in dirs.split(':') {
            check_path!(PathBuf::from(dir));
        }
    }

    if let Ok(prefix) = env::var("PREFIX") {
        let path = PathBuf::from(prefix);
        check_path!(path.join("etc/terminfo"));
        check_path!(path.join("lib/terminfo"));
        check_path!(path.join("share/terminfo"));
    }

    check_path!(PathBuf::from("/etc/terminfo"));
    check_path!(PathBuf::from("/lib/terminfo"));
    check_path!(PathBuf::from("/usr/share/terminfo"));
    check_path!(PathBuf::from("/boot/system/data/terminfo"));

    false
}
