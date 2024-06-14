pub struct Args {
    pub disable_validation: bool,
    pub window_protocol: Option<WindowProtocol>,
    pub command: Vec<String>,
    pub working_dir: Option<String>,
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

        Args {
            disable_validation: args
                .contains(&"--disable-validation".to_string()),
            window_protocol,
            command: args.into_iter().skip(1).collect(),
            working_dir: None,
        }
    }

    pub fn command(&self) -> Option<(&str, &[String])> {
        if self.command.is_empty() {
            return None;
        }

        Some((&self.command[0], &self.command[1..]))
    }
}
