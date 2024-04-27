use std::{
    cmp::max, io::Read, sync::{Arc, Mutex}, time::{Duration, Instant}
};

use filedescriptor::{poll, pollfd, POLLIN};
use portable_pty::{native_pty_system, unix::RawFd, CommandBuilder, MasterPty, PtySize};
use termwiz::escape::{
    csi::{DecPrivateMode, DecPrivateModeCode, Device, Mode},
    Action, CSI,
};
use wezterm_term::{color::ColorPalette, Alert, AlertHandler, Terminal, TerminalConfiguration, TerminalSize};

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
    title: Mutex<String>,
}

impl Pane {
    pub fn new(id: PaneId, size: TerminalSize) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(PtySize {
            rows: size.rows as u16,
            cols: size.cols as u16,
            pixel_width: size.pixel_width as u16,
            pixel_height: size.pixel_height as u16,
        })?;

        let cmd = CommandBuilder::new("zsh");
        pty.slave.spawn_command(cmd)?;
        let mut terminal = Terminal::new(
            size,
            Arc::new(TermConfig::new()),
            "Raven",
            "1.0.0",
            pty.master.take_writer()?,
        );

        terminal.set_notification_handler(Box::new(NotificationHandler { pane_id: id }));

        Ok(Self {
            id,
            terminal: Mutex::new(terminal),
            pty: Mutex::new(pty.master),
            title: Mutex::new(format!("Terminal #{}", id))
        })
    }

    pub fn title(&self) -> String {
        self.title.lock().unwrap().clone()
    }

    pub fn set_title(&self, title: String) {
        *self.title.lock().unwrap() = title;
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

    pub fn resize(&self, terminal_size: (f32, f32), cell_size: (f32, f32), row_spacing: usize) {
        let mut terminal = self.terminal()
                .lock()
                .expect("Unable to obtain terminal");

        let (terminal_width, terminal_height) = terminal_size;
        let (cell_width, cell_height) = cell_size;

        let cols = max((terminal_width / cell_width) as usize, 1);
        let rows = max((terminal_height / cell_height) as usize, 1);

        let total_row_spacing = row_spacing * rows;
        let terminal_height_with_row_spacing = terminal_height - total_row_spacing as f32;

        let rows = max((terminal_height_with_row_spacing / cell_height) as usize, 1);

        self.pty.lock().unwrap().resize(PtySize {
            rows: rows as u16,
            cols: cols as u16,
            pixel_width: terminal_width as u16,
            pixel_height: terminal_height as u16
        }).expect("Unable to resize pty");

        terminal.resize(TerminalSize {
            rows,
            cols,
            pixel_height: terminal_height as usize,
            pixel_width: terminal_width as usize,
            dpi: 1,
        });

        std::mem::drop(terminal);

        let events = Events::get();
        events.emit(Event::PaneOutput(self.id));
    }
}

impl PartialEq for Pane {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

struct NotificationHandler {
    pane_id: PaneId
}

impl AlertHandler for NotificationHandler {
    fn alert(&mut self, alert: wezterm_term::Alert) {
        let events = Events::get();
        match alert {
            Alert::TabTitleChanged(title) => {
                events.emit(Event::PaneTitle {
                    pane_id: self.pane_id,
                    title: title.unwrap_or(String::new())
                });
            }
            _ => {}
        }
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
        events.emit(Event::PaneOutput(pane.id));
    }

    if !actions.is_empty() {
        perform_actions(&pane, std::mem::take(&mut actions));
        events.emit(Event::PaneOutput(pane.id));
    }

    Ok(())
}
