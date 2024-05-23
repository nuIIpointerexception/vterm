use std::{
    env,
    ffi::CStr,
    fs::File,
    io::{Error, ErrorKind, Read, Result},
    mem::MaybeUninit,
    os::{
        fd::OwnedFd,
        unix::{
            io::{AsRawFd, FromRawFd},
            net::UnixStream,
            process::CommandExt,
        },
    },
    process::{Child, Command, Stdio},
    ptr,
    sync::Arc,
};

use libc::{c_int, TIOCSCTTY};
use log::error;
use polling::{Event, PollMode, Poller};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use rustix_openpty::rustix::termios::{self, InputModes, OptionalActions};
use rustix_openpty::{openpty, rustix::termios::Winsize};
use signal_hook::{consts as sigconsts, low_level::pipe as signal_pipe};

use crate::{
    event::{OnResize, WindowSize},
    tty::{ChildEvent, EventedPty, EventedReadWrite, Options},
};

pub(crate) const PTY_READ_WRITE_TOKEN: usize = 0;

pub(crate) const PTY_CHILD_EVENT_TOKEN: usize = 1;

macro_rules! die {
    ($($arg:tt)*) => {{
        error!($($arg)*);
        std::process::exit(1);
    }}
}

fn set_controlling_terminal(fd: c_int) {
    let res = unsafe {
        #[allow(clippy::cast_lossless)]
        libc::ioctl(fd, TIOCSCTTY as _, 0)
    };

    if res < 0 {
        die!("ioctl TIOCSCTTY failed: {}", Error::last_os_error());
    }
}

#[derive(Debug)]
struct Passwd<'a> {
    name: &'a str,
    dir: &'a str,
    shell: &'a str,
}

fn get_pw_entry(buf: &mut [i8; 1024]) -> Result<Passwd<'_>> {
    let mut entry: MaybeUninit<libc::passwd> = MaybeUninit::uninit();

    let mut res: *mut libc::passwd = ptr::null_mut();

    let uid = unsafe { libc::getuid() };
    let status = unsafe {
        libc::getpwuid_r(
            uid,
            entry.as_mut_ptr(),
            buf.as_mut_ptr() as *mut _,
            buf.len(),
            &mut res,
        )
    };
    let entry = unsafe { entry.assume_init() };

    if status < 0 {
        return Err(Error::new(ErrorKind::Other, "getpwuid_r failed"));
    }

    if res.is_null() {
        return Err(Error::new(ErrorKind::Other, "pw not found"));
    }

    assert_eq!(entry.pw_uid, uid);

    Ok(Passwd {
        name: unsafe { CStr::from_ptr(entry.pw_name).to_str().unwrap() },
        dir: unsafe { CStr::from_ptr(entry.pw_dir).to_str().unwrap() },
        shell: unsafe { CStr::from_ptr(entry.pw_shell).to_str().unwrap() },
    })
}

pub struct Pty {
    child: Child,
    file: File,
    signals: UnixStream,
}

impl Pty {
    pub fn child(&self) -> &Child {
        &self.child
    }

    pub fn file(&self) -> &File {
        &self.file
    }
}

struct ShellUser {
    user: String,
    home: String,
    shell: String,
}

impl ShellUser {
    fn from_env() -> Result<Self> {
        let mut buf = [0; 1024];
        let pw = get_pw_entry(&mut buf);

        let user = match env::var("USER") {
            Ok(user) => user,
            Err(_) => match pw {
                Ok(ref pw) => pw.name.to_owned(),
                Err(err) => return Err(err),
            },
        };

        let home = match env::var("HOME") {
            Ok(home) => home,
            Err(_) => match pw {
                Ok(ref pw) => pw.dir.to_owned(),
                Err(err) => return Err(err),
            },
        };

        let shell = match env::var("SHELL") {
            Ok(shell) => shell,
            Err(_) => match pw {
                Ok(ref pw) => pw.shell.to_owned(),
                Err(err) => return Err(err),
            },
        };

        Ok(Self { user, home, shell })
    }
}

#[cfg(not(target_os = "macos"))]
fn default_shell_command(shell: &str, _user: &str) -> Command {
    Command::new(shell)
}

#[cfg(target_os = "macos")]
fn default_shell_command(shell: &str, user: &str) -> Command {
    let shell_name = shell.rsplit('/').next().unwrap();

    let mut login_command = Command::new("/usr/bin/login");

    let exec = format!("exec -a -{} {}", shell_name, shell);

    login_command.args(["-flp", user, "/bin/zsh", "-fc", &exec]);
    login_command
}

pub fn new(
    config: &Options,
    window_size: WindowSize,
    window_id: u64,
) -> Result<Pty> {
    let pty = openpty(None, Some(&window_size.to_winsize()))?;
    let (master, slave) = (pty.controller, pty.user);
    from_fd(config, window_id, master, slave)
}

pub fn from_fd(
    config: &Options,
    window_id: u64,
    master: OwnedFd,
    slave: OwnedFd,
) -> Result<Pty> {
    let master_fd = master.as_raw_fd();
    let slave_fd = slave.as_raw_fd();

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Ok(mut termios) = termios::tcgetattr(&master) {
        termios.input_modes.set(InputModes::IUTF8, true);
        let _ = termios::tcsetattr(&master, OptionalActions::Now, &termios);
    }

    let user = ShellUser::from_env()?;

    let mut builder = if let Some(shell) = config.shell.as_ref() {
        let mut cmd = Command::new(&shell.program);
        cmd.args(shell.args.as_slice());
        cmd
    } else {
        default_shell_command(&user.shell, &user.user)
    };

    builder.stdin(unsafe { Stdio::from_raw_fd(slave_fd) });
    builder.stderr(unsafe { Stdio::from_raw_fd(slave_fd) });
    builder.stdout(unsafe { Stdio::from_raw_fd(slave_fd) });

    let window_id = window_id.to_string();
    builder.env("ALACRITTY_WINDOW_ID", &window_id);
    builder.env("USER", user.user);
    builder.env("HOME", user.home);
    builder.env("WINDOWID", window_id);
    for (key, value) in &config.env {
        builder.env(key, value);
    }

    unsafe {
        builder.pre_exec(move || {
            let err = libc::setsid();
            if err == -1 {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to set session id",
                ));
            }

            set_controlling_terminal(slave_fd);

            libc::close(slave_fd);
            libc::close(master_fd);

            libc::signal(libc::SIGCHLD, libc::SIG_DFL);
            libc::signal(libc::SIGHUP, libc::SIG_DFL);
            libc::signal(libc::SIGINT, libc::SIG_DFL);
            libc::signal(libc::SIGQUIT, libc::SIG_DFL);
            libc::signal(libc::SIGTERM, libc::SIG_DFL);
            libc::signal(libc::SIGALRM, libc::SIG_DFL);

            Ok(())
        });
    }

    if let Some(dir) = &config.working_directory {
        builder.current_dir(dir);
    }

    let signals = {
        let (sender, recv) = UnixStream::pair()?;

        signal_pipe::register(sigconsts::SIGCHLD, sender)?;
        recv.set_nonblocking(true)?;
        recv
    };

    match builder.spawn() {
        Ok(child) => {
            unsafe {
                set_nonblocking(master_fd);
            }

            Ok(Pty {
                child,
                file: File::from(master),
                signals,
            })
        }
        Err(err) => Err(Error::new(
            err.kind(),
            format!(
                "Failed to spawn command '{}': {}",
                builder.get_program().to_string_lossy(),
                err
            ),
        )),
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.child.id() as i32, libc::SIGHUP);
        }
        let _ = self.child.wait();
    }
}

impl EventedReadWrite for Pty {
    type Reader = File;
    type Writer = File;

    #[inline]
    unsafe fn register(
        &mut self,
        poll: &Arc<Poller>,
        mut interest: Event,
        poll_opts: PollMode,
    ) -> Result<()> {
        interest.key = PTY_READ_WRITE_TOKEN;
        unsafe {
            poll.add_with_mode(&self.file, interest, poll_opts)?;
        }

        unsafe {
            poll.add_with_mode(
                &self.signals,
                Event::readable(PTY_CHILD_EVENT_TOKEN),
                PollMode::Level,
            )
        }
    }

    #[inline]
    fn reregister(
        &mut self,
        poll: &Arc<Poller>,
        mut interest: Event,
        poll_opts: PollMode,
    ) -> Result<()> {
        interest.key = PTY_READ_WRITE_TOKEN;
        poll.modify_with_mode(&self.file, interest, poll_opts)?;

        poll.modify_with_mode(
            &self.signals,
            Event::readable(PTY_CHILD_EVENT_TOKEN),
            PollMode::Level,
        )
    }

    #[inline]
    fn deregister(&mut self, poll: &Arc<Poller>) -> Result<()> {
        poll.delete(&self.file)?;
        poll.delete(&self.signals)
    }

    #[inline]
    fn reader(&mut self) -> &mut File {
        &mut self.file
    }

    #[inline]
    fn writer(&mut self) -> &mut File {
        &mut self.file
    }
}

impl EventedPty for Pty {
    #[inline]
    fn next_child_event(&mut self) -> Option<ChildEvent> {
        let mut buf = [0u8; 1];
        if let Err(err) = self.signals.read(&mut buf) {
            if err.kind() != ErrorKind::WouldBlock {
                error!("Error reading from signal pipe: {}", err);
            }
            return None;
        }

        match self.child.try_wait() {
            Err(err) => {
                error!("Error checking child process termination: {}", err);
                None
            }
            Ok(None) => None,
            Ok(exit_status) => {
                Some(ChildEvent::Exited(exit_status.and_then(|s| s.code())))
            }
        }
    }
}

impl OnResize for Pty {
    fn on_resize(&mut self, window_size: WindowSize) {
        let win = window_size.to_winsize();

        let res = unsafe {
            libc::ioctl(
                self.file.as_raw_fd(),
                libc::TIOCSWINSZ,
                &win as *const _,
            )
        };

        if res < 0 {
            die!("ioctl TIOCSWINSZ failed: {}", Error::last_os_error());
        }
    }
}

pub trait ToWinsize {
    fn to_winsize(self) -> Winsize;
}

impl ToWinsize for WindowSize {
    fn to_winsize(self) -> Winsize {
        let ws_row = self.num_lines as libc::c_ushort;
        let ws_col = self.num_cols as libc::c_ushort;

        let ws_xpixel = ws_col * self.cell_width as libc::c_ushort;
        let ws_ypixel = ws_row * self.cell_height as libc::c_ushort;
        Winsize {
            ws_row,
            ws_col,
            ws_xpixel,
            ws_ypixel,
        }
    }
}

unsafe fn set_nonblocking(fd: c_int) {
    use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

    let res = fcntl(fd, F_SETFL, fcntl(fd, F_GETFL, 0) | O_NONBLOCK);
    assert_eq!(res, 0);
}

#[test]
fn test_get_pw_entry() {
    let mut buf: [i8; 1024] = [0; 1024];
    let _pw = get_pw_entry(&mut buf).unwrap();
}
