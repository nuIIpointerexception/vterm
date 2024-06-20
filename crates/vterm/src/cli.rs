use std::str::FromStr;

use log::LevelFilter;

pub struct Args {
    pub disable_validation: bool,
    pub window_protocol: Option<WindowProtocol>,
    pub command: Vec<String>,
    pub log: bool,
    pub log_level: LevelFilter,
}

pub enum WindowProtocol {
    Wayland,
    X11,
}

impl Args {
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let wayland = args.contains(&"--wayland".to_string());
        let x11 = args.contains(&"--x11".to_string());

        if wayland && x11 {
            panic!("Cannot specify both --wayland and --x11");
        }

        let window_protocol = match (wayland, x11) {
            (true, false) => Some(WindowProtocol::Wayland),
            (false, true) => Some(WindowProtocol::X11),
            _ => None,
        };

        let log_level = args
            .iter()
            .find_map(|arg| {
                if arg.starts_with("--log-level=") {
                    let level_str = arg.split('=').nth(1)?;
                    LevelFilter::from_str(level_str).ok()
                } else {
                    None
                }
            })
            .unwrap_or(LevelFilter::Error);

        Args {
            disable_validation: args.contains(&"--disable-validation".to_string()),
            window_protocol,
            command: args.clone().into_iter().skip(1).collect(),
            log: args.contains(&"--log".to_string()),
            log_level,
        }
    }

    pub fn command(&self) -> Option<(&str, &[String])> {
        if self.command.is_empty() {
            return None;
        }

        Some((&self.command[0], &self.command[1 ..]))
    }
}
