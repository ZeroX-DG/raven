use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use config::{Palette, RgbaColor};
use crossbeam::{
    channel::{unbounded, Receiver, Sender},
    select,
};
use filedescriptor::{poll, pollfd, POLLIN};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use termwiz::escape::{
    csi::{DecPrivateMode, DecPrivateModeCode, Device, Mode},
    Action, CSI,
};
use wezterm_term::{
    color::ColorPalette, CursorPosition, KeyCode, KeyModifiers, Terminal, TerminalConfiguration,
    TerminalSize,
};

use crate::rendering::{render_terminal, LineElement};

pub fn create_terminal(
    size: TerminalSize,
) -> anyhow::Result<(Sender<UserEvent>, Receiver<TerminalEvent>)> {
    let terminal_loop = TerminalLoop::new(size)?;

    let user_event_tx = terminal_loop.user_event_channel.0.clone();
    let terminal_event_rx = terminal_loop.terminal_event_channel.1.clone();

    std::thread::spawn(|| terminal_loop.run());

    Ok((user_event_tx, terminal_event_rx))
}

pub enum TerminalEvent {
    Redraw {
        lines: Vec<LineElement>,
        cursor: CursorPosition,
        scroll_top: usize,
    },
    Exit,
}

pub enum UserEvent {
    Resize(TerminalSize),
    Paste(String),
    Keydown(KeyCode, KeyModifiers),
    Scroll(f64),
}

pub struct TerminalExtraState {
    scroll_top: usize,
}

struct TerminalLoop {
    terminal: Terminal,
    pty: Box<dyn MasterPty + Send>,
    user_event_channel: (Sender<UserEvent>, Receiver<UserEvent>),
    terminal_event_channel: (Sender<TerminalEvent>, Receiver<TerminalEvent>),
    extra_state: TerminalExtraState,
}

impl TerminalLoop {
    pub fn new(size: TerminalSize) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(PtySize {
            rows: size.rows as u16,
            cols: size.cols as u16,
            pixel_width: size.pixel_width as u16,
            pixel_height: size.pixel_height as u16,
        })?;

        let shell = std::env::var("SHELL").unwrap_or(String::from("bash"));

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM_PROGRAM", "Raven");

        pty.slave.spawn_command(cmd)?;
        let terminal = Terminal::new(
            size,
            Arc::new(TermConfig::new()),
            "Raven",
            "1.0.0",
            pty.master.take_writer()?,
        );

        Ok(Self {
            terminal,
            pty: pty.master,
            user_event_channel: unbounded(),
            terminal_event_channel: unbounded(),
            extra_state: TerminalExtraState { scroll_top: 0 },
        })
    }

    fn handle_user_event(&mut self, event: UserEvent) -> anyhow::Result<()> {
        match event {
            UserEvent::Resize(size) => {
                self.pty
                    .resize(PtySize {
                        rows: size.rows as u16,
                        cols: size.cols as u16,
                        pixel_width: size.pixel_width as u16,
                        pixel_height: size.pixel_height as u16,
                    })
                    .unwrap();
                self.terminal.resize(size);
            }
            UserEvent::Paste(content) => {
                self.terminal.send_paste(&content)?;
            }
            UserEvent::Keydown(key, mods) => {
                self.terminal.key_down(key, mods)?;
            }
            UserEvent::Scroll(delta_y) => {
                let screen = self.terminal.screen();
                let max_offset = screen.scrollback_rows() - screen.physical_rows;
                let current_offset = self.extra_state.scroll_top as f64;
                let mut new_offset = current_offset + delta_y * 0.2;

                if new_offset < 0. {
                    new_offset = 0.;
                } else if new_offset as usize > max_offset {
                    new_offset = max_offset as f64;
                }

                self.extra_state.scroll_top = new_offset as usize;
            }
        }

        Ok(())
    }

    pub fn run(mut self) -> anyhow::Result<()> {
        let pty_read_thread = PtyReadThread::new(&self.pty);
        let terminal_actions_rx = pty_read_thread.actions();
        let user_event_rx = self.user_event_channel.1.clone();
        let terminal_event_tx = self.terminal_event_channel.0.clone();

        loop {
            select! {
                recv(terminal_actions_rx) -> actions => {
                    let Ok(actions) = actions else {
                        terminal_event_tx.send(TerminalEvent::Exit)?;
                        break;
                    };
                    self.terminal.perform_actions(actions);

                    let scroll_top = self.extra_state.scroll_top;

                    let (lines, cursor) = render_terminal(&self.terminal, scroll_top);
                    terminal_event_tx.send(TerminalEvent::Redraw { lines, cursor, scroll_top })?;
                }
                recv(user_event_rx) -> event => {
                    let Ok(event) = event else {
                        break;
                    };
                    self.handle_user_event(event)?;
                }
            }
        }

        pty_read_thread.close();
        Ok(())
    }
}

struct PtyReadThread {
    thread: std::thread::JoinHandle<()>,
    actions_rx: Receiver<Vec<Action>>,
}

impl PtyReadThread {
    pub fn new(pty: &Box<dyn MasterPty + Send>) -> Self {
        let mut reader = pty.try_clone_reader().unwrap();
        let pty_raw_fd = pty.as_raw_fd().unwrap();
        let (tx, rx) = unbounded();

        let thread = std::thread::spawn(move || {
            let delay = Duration::from_millis(3);
            let mut buf = vec![0u8; 128 * 1024];

            let mut parser = termwiz::escape::parser::Parser::new();
            let mut actions = vec![];
            let mut action_size = 0;
            let mut hold = false;
            let mut deadline = None;

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
                                Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(
                                    DecPrivateMode::Code(DecPrivateModeCode::SynchronizedOutput),
                                ))) => {
                                    hold = true;
                                    // Flush prior actions
                                    if !actions.is_empty() {
                                        tx.send(std::mem::take(&mut actions)).unwrap();
                                        action_size = 0;
                                    }
                                }
                                Action::CSI(CSI::Mode(Mode::ResetDecPrivateMode(
                                    DecPrivateMode::Code(DecPrivateModeCode::SynchronizedOutput),
                                ))) => {
                                    hold = false;
                                    flush = true;
                                }
                                Action::CSI(CSI::Device(dev))
                                    if matches!(**dev, Device::SoftReset) =>
                                {
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

                            tx.send(std::mem::take(&mut actions)).unwrap();
                            deadline = None;
                            action_size = 0;
                        }
                    }
                }
            }

            if !actions.is_empty() {
                tx.send(std::mem::take(&mut actions)).unwrap();
            }
        });

        Self {
            thread,
            actions_rx: rx,
        }
    }

    pub fn actions(&self) -> &Receiver<Vec<Action>> {
        &self.actions_rx
    }

    pub fn close(self) {
        let _ = self.thread.join();
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

    fn color_palette(&self) -> ColorPalette {
        ColorPalette::from(Palette {
            foreground: Some(RgbaColor::try_from("#cbccc6".to_string()).unwrap()),
            // Modified to match with app background
            background: Some(RgbaColor::try_from("rgb(17, 21, 28)".to_string()).unwrap()),
            cursor_fg: Some(RgbaColor::try_from("#1f2430".to_string()).unwrap()),
            cursor_bg: Some(RgbaColor::try_from("#ffcc66".to_string()).unwrap()),
            cursor_border: Some(RgbaColor::try_from("#ffcc66".to_string()).unwrap()),
            selection_fg: Some(RgbaColor::try_from("#cbccc6".to_string()).unwrap()),
            selection_bg: Some(RgbaColor::try_from("#33415e".to_string()).unwrap()),
            ansi: Some([
                RgbaColor::try_from("#191e2a".to_string()).unwrap(),
                RgbaColor::try_from("#ed8274".to_string()).unwrap(),
                RgbaColor::try_from("#a6cc70".to_string()).unwrap(),
                RgbaColor::try_from("#fad07b".to_string()).unwrap(),
                RgbaColor::try_from("#6dcbfa".to_string()).unwrap(),
                RgbaColor::try_from("#cfbafa".to_string()).unwrap(),
                RgbaColor::try_from("#90e1c6".to_string()).unwrap(),
                RgbaColor::try_from("#c7c7c7".to_string()).unwrap(),
            ]),
            brights: Some([
                RgbaColor::try_from("#686868".to_string()).unwrap(),
                RgbaColor::try_from("#f28779".to_string()).unwrap(),
                RgbaColor::try_from("#bae67e".to_string()).unwrap(),
                RgbaColor::try_from("#ffd580".to_string()).unwrap(),
                RgbaColor::try_from("#73d0ff".to_string()).unwrap(),
                RgbaColor::try_from("#d4bfff".to_string()).unwrap(),
                RgbaColor::try_from("#95e6cb".to_string()).unwrap(),
                RgbaColor::try_from("#ffffff".to_string()).unwrap(),
            ]),
            ..Default::default()
        })
    }
}
