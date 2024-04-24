use std::{
    io::Read,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use filedescriptor::{poll, pollfd, POLLIN};
use portable_pty::{native_pty_system, unix::RawFd, CommandBuilder, MasterPty, PtySize};
use termwiz::escape::{
    csi::{DecPrivateMode, DecPrivateModeCode, Device, Mode},
    Action, CSI,
};
use wezterm_term::{color::ColorPalette, Terminal, TerminalConfiguration, TerminalSize};

use crate::events::{Event, Events};

pub type PaneId = usize;
static PANE_ID: ::std::sync::atomic::AtomicUsize = ::std::sync::atomic::AtomicUsize::new(0);

pub fn alloc_pane_id() -> PaneId {
    PANE_ID.fetch_add(1, ::std::sync::atomic::Ordering::Relaxed)
}

pub struct Pane {
    pub id: PaneId,
    terminal: Mutex<Terminal>,
    pty: Mutex<Box<dyn MasterPty + Send>>,
}

impl Pane {
    pub fn new(id: PaneId, size: TerminalSize) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let cmd = CommandBuilder::new("bash");
        pty.slave.spawn_command(cmd)?;
        let terminal = Terminal::new(
            size,
            Arc::new(TermConfig::new()),
            "Raven",
            "1.0.0",
            pty.master.take_writer()?,
        );

        Ok(Self {
            id,
            terminal: Mutex::new(terminal),
            pty: Mutex::new(pty.master),
        })
    }

    pub fn reader(&self) -> anyhow::Result<Box<dyn Read + Send>> {
        self.pty
            .lock()
            .map_err(|_| anyhow::Error::msg("Unable to obtain pty"))?
            .try_clone_reader()
    }

    pub fn as_raw_fd(&self) -> anyhow::Result<Option<RawFd>> {
        let pty = self
            .pty
            .lock()
            .map_err(|_| anyhow::Error::msg("Unable to obtain pty"))?;
        Ok(pty.as_raw_fd())
    }

    pub fn terminal(&self) -> &Mutex<Terminal> {
        &self.terminal
    }
}

impl PartialEq for Pane {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug)]
struct TermConfig {
    scrollback: usize,
}

impl TermConfig {
    pub fn new() -> Self {
        Self { scrollback: 1000 }
    }
}

impl TerminalConfiguration for TermConfig {
    fn scrollback_size(&self) -> usize {
        self.scrollback
    }

    fn color_palette(&self) -> wezterm_term::color::ColorPalette {
        ColorPalette::default()
    }
}

fn perform_actions(pane: &Arc<Pane>, actions: Vec<Action>) {
    pane.terminal()
        .lock()
        .expect("Unable to obtain terminal")
        .perform_actions(actions);
}

pub fn read_from_pane_pty(pane: Arc<Pane>) -> anyhow::Result<()> {
    let delay = Duration::from_millis(3);
    let pty_raw_fd = pane
        .as_raw_fd()?
        .ok_or(anyhow::Error::msg("Unable to obtain pty raw fd"))?;
    let mut reader = pane.reader()?;
    let mut buf = vec![0u8; 128 * 1024];

    let mut parser = termwiz::escape::parser::Parser::new();
    let mut actions = vec![];
    let mut action_size = 0;
    let mut hold = false;
    let mut deadline = None;

    let events = Events::get();

    loop {
        match reader.read(&mut buf) {
            Ok(size) if size == 0 => {
                break;
            }
            Err(_) => {
                break;
            }
            Ok(size) => {
                parser.parse(&buf[0..size], |action| {
                    let mut flush = false;
                    match &action {
                        Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                            DecPrivateModeCode::SynchronizedOutput,
                        )))) => {
                            hold = true;
                            // Flush prior actions
                            if !actions.is_empty() {
                                perform_actions(&pane, std::mem::take(&mut actions));
                                action_size = 0;
                            }
                        }
                        Action::CSI(CSI::Mode(Mode::ResetDecPrivateMode(
                            DecPrivateMode::Code(DecPrivateModeCode::SynchronizedOutput),
                        ))) => {
                            hold = false;
                            flush = true;
                        }
                        Action::CSI(CSI::Device(dev)) if matches!(**dev, Device::SoftReset) => {
                            hold = false;
                            flush = true;
                        }
                        _ => {}
                    };
                    action.append_to(&mut actions);
                    if flush && !actions.is_empty() {
                        action_size = 0;
                    }
                });
                action_size += size;
                if !actions.is_empty() && !hold {
                    // If we haven't accumulated too much data,
                    // pause for a short while to increase the chances
                    // that we coalesce a full "frame" from an unoptimized
                    // TUI program
                    if action_size < buf.len() {
                        let poll_delay = match deadline {
                            None => {
                                deadline.replace(Instant::now() + delay);
                                Some(delay)
                            }
                            Some(target) => target.checked_duration_since(Instant::now()),
                        };
                        if poll_delay.is_some() {
                            let mut pfd = [pollfd {
                                fd: pty_raw_fd,
                                events: POLLIN,
                                revents: 0,
                            }];
                            if let Ok(1) = poll(&mut pfd, poll_delay) {
                                // We can read now without blocking, so accumulate
                                // more data into actions
                                continue;
                            }

                            // Not readable in time: let the data we have flow into
                            // the terminal model
                        }
                    }

                    perform_actions(&pane, std::mem::take(&mut actions));
                    deadline = None;
                    action_size = 0;
                }
            }
        }
        events.emit(Event::OutputUpdate(pane.id));
    }

    if !actions.is_empty() {
        perform_actions(&pane, std::mem::take(&mut actions));
        events.emit(Event::OutputUpdate(pane.id));
    }

    Ok(())
}
