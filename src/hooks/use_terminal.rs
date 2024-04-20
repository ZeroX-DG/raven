use std::rc::Rc;

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use freya::prelude::*;

pub type SessionId = usize;

#[derive(Clone)]
pub struct Session {
    id: SessionId,
    reader: Rc<Box<dyn std::io::Read + Send>>,
    writer: Rc<Box<dyn std::io::Write + Send>>
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

        let reader = Rc::new(pty.master.try_clone_reader()?);
        let writer = Rc::new(pty.master.take_writer()?);

        Ok(Self { id, reader, writer })
    }
}

impl PartialEq for Session {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
 
pub struct UseTerminal {
    pub(crate) active_session: Memo<Option<Session>>,
}

impl UseTerminal {
    pub fn active_session(&self) -> Memo<Option<Session>> {
        self.active_session
    }
}

pub fn use_terminal<'a>() -> UseTerminal {
    let sessions = use_signal::<Vec<Session>>(|| Vec::new());
    let active_session_id = use_signal::<Option<SessionId>>(|| None);

    let active_session = use_memo::<Option<Session>>(move || {
        active_session_id.read()
            .map(|id| sessions.read().get(id).map(|session| session.clone()))
            .flatten()
    });

    UseTerminal { active_session }
}