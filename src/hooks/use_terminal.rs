use std::{sync::Arc, thread, time::{Duration, Instant}};

use portable_pty::{native_pty_system, CommandBuilder, PtyPair, PtySize};
use freya::prelude::*;
use termwiz::escape::{csi::{DecPrivateMode, DecPrivateModeCode, Device, Mode}, Action, CSI};
use wezterm_term::{color::ColorPalette, Terminal, TerminalConfiguration, TerminalSize};
use filedescriptor::{poll, pollfd, POLLIN};

pub type SessionId = usize;

pub struct Session {
    id: SessionId,
    terminal: Terminal,
    pty: PtyPair,
    pty_as_raw_fd: i32
}

#[derive(Clone, Debug)]
pub struct LineElement {
    segments: Vec<LineSegment>
}

#[derive(Clone, Debug)]
pub struct LineSegment {
    pub text: String
}

impl LineElement {
    pub fn segments(&self) -> &Vec<LineSegment> {
        &self.segments
    }
}

#[derive(Debug)]
struct RavenTermConfig {
    scrollback: usize
}

impl TerminalConfiguration for RavenTermConfig {
    fn scrollback_size(&self) -> usize {
        self.scrollback
    }

    fn color_palette(&self) -> wezterm_term::color::ColorPalette {
        ColorPalette::default()
    }
}

impl Session {
    pub fn new(id: SessionId) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();
        let pty = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0
        })?;
        let raw_fd = pty.master.as_raw_fd()
            .ok_or(anyhow::Error::msg("Unable to obtain pty rawfd"))?;

        let cmd = CommandBuilder::new("bash");
        pty.slave.spawn_command(cmd)?;
        let terminal = Terminal::new(
            TerminalSize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
                dpi: 1
            },
            Arc::new(RavenTermConfig { scrollback: 100 }),
            "Raven",
            "1.0.0",
            pty.master.take_writer()?
        );

        Ok(Self { id, terminal, pty, pty_as_raw_fd: raw_fd })
    }

    pub fn get_lines(&self) -> Vec<LineElement> {
        let mut lines = vec![];
        self.terminal.screen().for_each_phys_line(|_, line| {
            let segments = line.cluster(None).iter()
                .map(|cluster| {
                    LineSegment {
                        text: cluster.text.clone()
                    }
                })
                .collect();

            lines.push(LineElement {
                segments
            })
        });
        lines
    }
}

pub struct UseTerminal {
    active_session_lines: SyncSignal<Vec<LineElement>>,
}

impl UseTerminal {
    pub fn active_session_lines(&self) -> SyncSignal<Vec<LineElement>> {
        self.active_session_lines
    }
}

pub fn use_terminal() -> UseTerminal {
    let mut active_session_lines = use_signal_sync::<Vec<LineElement>>(|| Vec::new());

    use_hook(|| {
        thread::spawn(move || {
            let mut session = Session::new(0).unwrap();
            let mut reader = session.pty.master.try_clone_reader().unwrap();
            let mut parser = termwiz::escape::parser::Parser::new();
            let mut buf = vec![0u8; 128 * 1024];
            let mut actions = vec![];
            let mut action_size = 0;
            let mut hold = false;
            let delay = Duration::from_millis(3);
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
                                Action::CSI(CSI::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
                                    DecPrivateModeCode::SynchronizedOutput,
                                )))) => {
                                    hold = true;
                                    // Flush prior actions
                                    if !actions.is_empty() {
                                        session.terminal.perform_actions(std::mem::take(&mut actions));
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
                                session.terminal.perform_actions(std::mem::take(&mut actions));
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
                                        fd: session.pty_as_raw_fd,
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

                            session.terminal.perform_actions(std::mem::take(&mut actions));
                            deadline = None;
                            action_size = 0;
                        } 
                    }
                }
                let lines = session.get_lines();
                active_session_lines.set(lines);
            }
        });
    });

    UseTerminal { active_session_lines }
}