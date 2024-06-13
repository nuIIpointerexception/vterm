pub struct Args {
    pub disable_validation: bool,
    pub window_protocol: Option<WindowProtocol>,
}

pub enum WindowProtocol {
    Wayland,
    X11,
}

// TODO(nuii): reimplement validation layers and make this work properly.
impl Args {
    pub fn parse() -> Args {
        let wayland = std::env::args().any(|arg| arg == "--wayland");
        let x11 = std::env::args().any(|arg| arg == "--x11");
        let window_protocol = if wayland && x11 {
            panic!("can't specify both --wayland and --x11");
        } else if wayland {
            Some(WindowProtocol::Wayland)
        } else if x11 {
            Some(WindowProtocol::X11)
        } else {
            None
        };
        Args {
            disable_validation: std::env::args()
                .any(|arg| arg == "--disable-validation"),
            window_protocol,
        }
    }
}
