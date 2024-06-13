use std::{
    borrow::Cow,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, ErrorKind, Read, Write},
    num::NonZeroUsize,
    sync::{
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc,
    },
    thread::JoinHandle,
    time::Instant,
};

use log::error;
use polling::{Event as PollingEvent, Events, PollMode};

use crate::{
    event::{self, Event, EventListener, WindowSize},
    sync::FairMutex,
    term::Term,
    thread, tty,
    vte::ansi,
};

pub(crate) const READ_BUFFER_SIZE: usize = 0x10_0000;

const MAX_LOCKED_READ: usize = u16::MAX as usize;

#[derive(Debug)]
pub enum Msg {
    Input(Cow<'static, [u8]>),

    Shutdown,

    Resize(WindowSize),
}

pub struct EventLoop<T: tty::EventedPty, U: EventListener> {
    poll: Arc<polling::Poller>,
    pty: T,
    rx: PeekableReceiver<Msg>,
    tx: Sender<Msg>,
    terminal: Arc<FairMutex<Term<U>>>,
    event_proxy: U,
    hold: bool,
    ref_test: bool,
}

impl<T, U> EventLoop<T, U>
    where
        T: tty::EventedPty + event::OnResize + Send + 'static,
        U: EventListener + Send + 'static,
{
    pub fn new(
        terminal: Arc<FairMutex<Term<U>>>,
        event_proxy: U,
        pty: T,
        hold: bool,
        ref_test: bool,
    ) -> io::Result<EventLoop<T, U>> {
        let (tx, rx) = mpsc::channel();
        let poll = polling::Poller::new()?.into();
        Ok(EventLoop {
            poll,
            pty,
            tx,
            rx: PeekableReceiver::new(rx),
            terminal,
            event_proxy,
            hold,
            ref_test,
        })
    }

    pub fn channel(&self) -> EventLoopSender {
        EventLoopSender {
            sender: self.tx.clone(),
            poller: self.poll.clone(),
        }
    }

    fn drain_recv_channel(&mut self, state: &mut State) -> bool {
        while let Some(msg) = self.rx.recv() {
            match msg {
                Msg::Input(input) => state.write_list.push_back(input),
                Msg::Resize(window_size) => self.pty.on_resize(window_size),
                Msg::Shutdown => return false,
            }
        }

        true
    }

    #[inline]
    fn pty_read<X>(
        &mut self,
        state: &mut State,
        buf: &mut [u8],
        mut writer: Option<&mut X>,
    ) -> io::Result<()>
        where
            X: Write,
    {
        let mut unprocessed = 0;
        let mut processed = 0;

        let _terminal_lease = Some(self.terminal.lease());
        let mut terminal = None;

        loop {
            match self.pty.reader().read(&mut buf[unprocessed..]) {
                Ok(0) if unprocessed == 0 => break,
                Ok(got) => unprocessed += got,
                Err(err) => match err.kind() {
                    ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                        if unprocessed == 0 {
                            break;
                        }
                    }
                    _ => return Err(err),
                },
            }

            let terminal = match &mut terminal {
                Some(terminal) => terminal,
                None => {
                    terminal.insert(match self.terminal.try_lock_unfair() {
                        None if unprocessed >= READ_BUFFER_SIZE => {
                            self.terminal.lock_unfair()
                        }
                        None => continue,
                        Some(terminal) => terminal,
                    })
                }
            };

            if let Some(writer) = &mut writer {
                writer.write_all(&buf[..unprocessed]).unwrap();
            }

            for byte in &buf[..unprocessed] {
                state.parser.advance(&mut **terminal, *byte);
            }

            processed += unprocessed;
            unprocessed = 0;

            if processed >= MAX_LOCKED_READ {
                break;
            }
        }

        if state.parser.sync_bytes_count() < processed && processed > 0 {
            self.event_proxy.send_event(Event::Wakeup);
        }

        Ok(())
    }

    #[inline]
    fn pty_write(&mut self, state: &mut State) -> io::Result<()> {
        state.ensure_next();

        'write_many: while let Some(mut current) = state.take_current() {
            'write_one: loop {
                match self.pty.writer().write(current.remaining_bytes()) {
                    Ok(0) => {
                        state.set_current(Some(current));
                        break 'write_many;
                    }
                    Ok(n) => {
                        current.advance(n);
                        if current.finished() {
                            state.goto_next();
                            break 'write_one;
                        }
                    }
                    Err(err) => {
                        state.set_current(Some(current));
                        match err.kind() {
                            ErrorKind::Interrupted | ErrorKind::WouldBlock => {
                                break 'write_many;
                            }
                            _ => return Err(err),
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn spawn(mut self) -> JoinHandle<(Self, State)> {
        thread::spawn_named("PTY reader", move || {
            let mut state = State::default();
            let mut buf = [0u8; READ_BUFFER_SIZE];

            let poll_opts = PollMode::Level;
            let mut interest = PollingEvent::readable(0);

            if let Err(err) =
                unsafe { self.pty.register(&self.poll, interest, poll_opts) }
            {
                error!("Event loop registration error: {}", err);
                return (self, state);
            }

            let mut events =
                Events::with_capacity(NonZeroUsize::new(1024).unwrap());

            let mut pipe = if self.ref_test {
                Some(
                    File::create("./alacritty.recording")
                        .expect("create alacritty recording"),
                )
            } else {
                None
            };

            'event_loop: loop {
                let handler = state.parser.sync_timeout();
                let timeout = handler
                    .sync_timeout()
                    .map(|st| st.saturating_duration_since(Instant::now()));

                events.clear();
                if let Err(err) = self.poll.wait(&mut events, timeout) {
                    match err.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => {
                            error!("Event loop polling error: {}", err);
                            break 'event_loop;
                        }
                    }
                }

                if events.is_empty() && self.rx.peek().is_none() {
                    state.parser.stop_sync(&mut *self.terminal.lock());
                    self.event_proxy.send_event(Event::Wakeup);
                    continue;
                }

                if !self.drain_recv_channel(&mut state) {
                    break;
                }

                for event in events.iter() {
                    match event.key {
                        tty::PTY_CHILD_EVENT_TOKEN => {
                            if let Some(tty::ChildEvent::Exited(code)) =
                                self.pty.next_child_event()
                            {
                                if let Some(code) = code {
                                    self.event_proxy
                                        .send_event(Event::ChildExit(code));
                                }
                                if self.hold {
                                    let _ = self.pty_read(
                                        &mut state,
                                        &mut buf,
                                        pipe.as_mut(),
                                    );
                                } else {
                                    self.terminal.lock().exit();
                                }
                                self.event_proxy.send_event(Event::Wakeup);
                                break 'event_loop;
                            }
                        }

                        tty::PTY_READ_WRITE_TOKEN => {
                            if event.is_interrupt() {
                                continue;
                            }

                            if event.readable {
                                if let Err(err) = self.pty_read(
                                    &mut state,
                                    &mut buf,
                                    pipe.as_mut(),
                                ) {
                                    #[cfg(target_os = "linux")]
                                    if err.raw_os_error() == Some(libc::EIO) {
                                        continue;
                                    }

                                    error!("Error reading from PTY in event loop: {}", err);
                                    break 'event_loop;
                                }
                            }

                            if event.writable {
                                if let Err(err) = self.pty_write(&mut state) {
                                    error!("Error writing to PTY in event loop: {}", err);
                                    break 'event_loop;
                                }
                            }
                        }
                        _ => (),
                    }
                }

                let needs_write = state.needs_write();
                if needs_write != interest.writable {
                    interest.writable = needs_write;

                    self.pty
                        .reregister(&self.poll, interest, poll_opts)
                        .unwrap();
                }
            }

            let _ = self.pty.deregister(&self.poll);

            (self, state)
        })
    }
}

struct Writing {
    source: Cow<'static, [u8]>,
    written: usize,
}

pub struct Notifier(pub EventLoopSender);

impl event::Notify for Notifier {
    fn notify<B>(&self, bytes: B)
        where
            B: Into<Cow<'static, [u8]>>,
    {
        let bytes = bytes.into();
        if bytes.len() == 0 {
            return;
        }

        let _ = self.0.send(Msg::Input(bytes));
    }
}

impl event::OnResize for Notifier {
    fn on_resize(&mut self, window_size: WindowSize) {
        let _ = self.0.send(Msg::Resize(window_size));
    }
}

#[derive(Debug)]
pub enum EventLoopSendError {
    Io(io::Error),

    Send(mpsc::SendError<Msg>),
}

impl Display for EventLoopSendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EventLoopSendError::Io(err) => err.fmt(f),
            EventLoopSendError::Send(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for EventLoopSendError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EventLoopSendError::Io(err) => err.source(),
            EventLoopSendError::Send(err) => err.source(),
        }
    }
}

#[derive(Clone)]
pub struct EventLoopSender {
    sender: Sender<Msg>,
    poller: Arc<polling::Poller>,
}

impl EventLoopSender {
    pub fn send(&self, msg: Msg) -> Result<(), EventLoopSendError> {
        self.sender.send(msg).map_err(EventLoopSendError::Send)?;
        self.poller.notify().map_err(EventLoopSendError::Io)
    }
}

#[derive(Default)]
pub struct State {
    write_list: VecDeque<Cow<'static, [u8]>>,
    writing: Option<Writing>,
    parser: ansi::Processor,
}

impl State {
    #[inline]
    fn ensure_next(&mut self) {
        if self.writing.is_none() {
            self.goto_next();
        }
    }

    #[inline]
    fn goto_next(&mut self) {
        self.writing = self.write_list.pop_front().map(Writing::new);
    }

    #[inline]
    fn take_current(&mut self) -> Option<Writing> {
        self.writing.take()
    }

    #[inline]
    fn needs_write(&self) -> bool {
        self.writing.is_some() || !self.write_list.is_empty()
    }

    #[inline]
    fn set_current(&mut self, new: Option<Writing>) {
        self.writing = new;
    }
}

impl Writing {
    #[inline]
    fn new(c: Cow<'static, [u8]>) -> Writing {
        Writing {
            source: c,
            written: 0,
        }
    }

    #[inline]
    fn advance(&mut self, n: usize) {
        self.written += n;
    }

    #[inline]
    fn remaining_bytes(&self) -> &[u8] {
        &self.source[self.written..]
    }

    #[inline]
    fn finished(&self) -> bool {
        self.written >= self.source.len()
    }
}

struct PeekableReceiver<T> {
    rx: Receiver<T>,
    peeked: Option<T>,
}

impl<T> PeekableReceiver<T> {
    fn new(rx: Receiver<T>) -> Self {
        Self { rx, peeked: None }
    }

    fn peek(&mut self) -> Option<&T> {
        if self.peeked.is_none() {
            self.peeked = self.rx.try_recv().ok();
        }

        self.peeked.as_ref()
    }

    fn recv(&mut self) -> Option<T> {
        if self.peeked.is_some() {
            self.peeked.take()
        } else {
            match self.rx.try_recv() {
                Err(TryRecvError::Disconnected) => {
                    panic!("event loop channel closed")
                }
                res => res.ok(),
            }
        }
    }
}