use std::{io::{BufRead, BufReader}, rc::Rc, sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex}};

use anyhow::Error;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use freya::prelude::*;

pub type SessionId = usize;

struct Threads {
    reader: std::thread::JoinHandle<anyhow::Result<()>>,
    writer: std::thread::JoinHandle<anyhow::Result<()>>,
}

pub enum Change {
    Text(String)
}

pub struct Session {
    id: SessionId,
    pub writer: Sender<String>,
    pub reader: Arc<Mutex<Receiver<Change>>>,
    threads: Threads
}

impl Change {
    pub fn read(input: &mut impl BufRead) -> Result<Change, anyhow::Error> {
        let mut line = String::new();
        input.read_line(&mut line)?;
        Ok(Change::Text(line))
    }
}

impl Threads {
    pub fn join(self) -> anyhow::Result<()> {
        match self.reader.join() {
            Ok(r) => r?,
            Err(_) => Err(Error::msg("Reader panicked"))?
        };
        match self.writer.join() {
            Ok(r) => r?,
            Err(_) => Err(Error::msg("Writer panicked"))?
        };
        Ok(())
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

        let cmd = CommandBuilder::new("bash");
        pty.slave.spawn_command(cmd)?;

        let pty_reader = pty.master.try_clone_reader()?;
        let mut pty_writer = pty.master.take_writer()?;

        let (writer_tx, writer_rx) = channel::<String>();
        let (reader_tx, reader_rx) = channel::<Change>();

        let writer = std::thread::spawn(move || loop {
            let content = writer_rx.recv()?;
            pty_writer.write_all(content.as_bytes())?;
        });

        let reader = std::thread::spawn(move || {
            let mut buf_read = BufReader::new(pty_reader);
            loop {
                match Change::read(&mut buf_read) {
                    Ok(change) => {
                        reader_tx.send(change)?;
                    }
                    Err(e) => {
                        println!("{:?}", e);
                    }
                }
            }
        });

        let threads = Threads { writer, reader };

        Ok(Self { id, writer: writer_tx, reader: Arc::new(Mutex::new(reader_rx)), threads })
    }

    pub fn write(&self, value: &str) {
        self.writer.send(value.to_string()).expect("unable to write");
    }

    pub fn close(self) -> anyhow::Result<()> {
        self.threads.join()
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct UseTerminal {
    sessions: Signal<Vec<Rc<Session>>>,
    pub(crate) active_session: Memo<Option<Rc<Session>>>,
}

impl UseTerminal {
    pub fn active_session(&self) -> Memo<Option<Rc<Session>>> {
        self.active_session
    }

    pub fn close_session(&mut self, session_id: SessionId) -> anyhow::Result<()> {
        let session = self.sessions.remove(session_id);
        match Rc::try_unwrap(session) {
            Ok(session) => session.close(),
            Err(_) => Err(anyhow::Error::msg("Unable to close session"))
        }
    }
}

pub fn use_terminal<'a>() -> UseTerminal {
    let sessions = use_signal::<Vec<Rc<Session>>>(|| vec![
        Rc::new(Session::new(0).unwrap())
    ]);
    let active_session_id = use_signal::<Option<SessionId>>(|| Some(0));

    let active_session = use_memo::<Option<Rc<Session>>>(move || {
        active_session_id.read()
            .map(|id| sessions.read().get(id).map(|session| session.clone()))
            .flatten()
    });

    UseTerminal { active_session, sessions }
}