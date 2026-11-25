use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, SyncSender},
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};

use crate::{paths::write_json, settings::write_settings_snapshot, state::AppState};

const DELAY: Duration = Duration::from_millis(400);

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Settings,
    History,
    Cache,
}

const KINDS: [Kind; 3] = [Kind::Settings, Kind::History, Kind::Cache];

#[derive(Default)]
struct Debounce {
    dirty: [bool; KINDS.len()],
    deadlines: [Option<Instant>; KINDS.len()],
}

impl Debounce {
    fn mark(&mut self, kind: Kind, at: Instant) {
        self.dirty[kind as usize] = true;
        self.deadlines[kind as usize] = Some(at + DELAY);
    }

    fn next(&self) -> Option<Instant> {
        self.deadlines.iter().flatten().copied().min()
    }

    fn write_ready(&mut self, through: Option<Instant>, mut write: impl FnMut(Kind) -> bool) {
        for kind in KINDS {
            let index = kind as usize;
            let ready = through.is_none_or(|now| self.deadlines[index].is_some_and(|at| at <= now));
            if self.dirty[index] && ready {
                self.deadlines[index] = None;
                self.dirty[index] = !write(kind);
            }
        }
    }
}

enum Message {
    Dirty(Kind, Instant),
    Flush(SyncSender<()>),
    Stop,
}

pub(crate) struct Writer(Sender<Message>);

impl Writer {
    pub(crate) fn start(app: AppHandle) -> Self {
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || run(receiver, |kind| write_snapshot(&app, kind)));
        Self(sender)
    }
}

fn run(receiver: Receiver<Message>, mut write: impl FnMut(Kind) -> bool) {
    let mut pending = Debounce::default();
    loop {
        let next = match pending.next() {
            Some(at) => receiver.recv_timeout(at.saturating_duration_since(Instant::now())),
            None => receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next {
            Ok(Message::Dirty(kind, at)) => {
                pending.mark(kind, at);
                pending.write_ready(Some(Instant::now()), &mut write);
            }
            Ok(Message::Flush(done)) => {
                pending.write_ready(None, &mut write);
                let _ = done.send(());
            }
            Ok(Message::Stop) | Err(RecvTimeoutError::Disconnected) => {
                pending.write_ready(None, &mut write);
                break;
            }
            Err(RecvTimeoutError::Timeout) => pending.write_ready(Some(Instant::now()), &mut write),
        }
    }
}

fn write_snapshot(app: &AppHandle, kind: Kind) -> bool {
    let state = app.state::<AppState>();
    match kind {
        Kind::Settings => write_settings_snapshot(&state).is_ok(),
        Kind::History => {
            let snapshot = state
                .history
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            write_json(&state.dir.join("history.json"), &snapshot).is_ok()
        }
        Kind::Cache => {
            let snapshot = state
                .cache
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .snapshot();
            write_json(&state.dir.join("cache.json"), &snapshot).is_ok()
        }
    }
}

pub(crate) fn schedule(app: &AppHandle, kind: Kind) {
    let state = app.state::<AppState>();
    if state
        .writer
        .0
        .send(Message::Dirty(kind, Instant::now()))
        .is_err()
    {
        eprintln!("Could not schedule Lexo data for writing");
    }
}

pub(crate) fn flush(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (done, wait) = mpsc::sync_channel(0);
    if state.writer.0.send(Message::Flush(done)).is_err() || wait.recv().is_err() {
        eprintln!("Could not flush Lexo data");
    }
}

pub(crate) fn stop(app: &AppHandle) {
    let state = app.state::<AppState>();
    let _ = state.writer.0.send(Message::Stop);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_marks_delay_one_write_per_kind() {
        let start = Instant::now();
        let mut pending = Debounce::default();
        let mut writes = Vec::new();
        let mut write = |kind| {
            writes.push(kind as usize);
            true
        };

        pending.mark(Kind::Settings, start);
        pending.mark(Kind::Cache, start);
        pending.mark(Kind::Settings, start + Duration::from_millis(300));
        pending.write_ready(Some(start + DELAY), &mut write);
        pending.write_ready(Some(start + DELAY * 2), &mut write);
        pending.write_ready(None, &mut write);

        assert_eq!(writes, [Kind::Cache as usize, Kind::Settings as usize]);
    }

    #[test]
    fn flush_waits_for_a_pending_write_without_waiting_for_debounce() {
        let (sender, receiver) = mpsc::channel();
        let (done, wait) = mpsc::sync_channel(0);
        let worker = thread::spawn(move || {
            let mut writes = 0;
            run(receiver, |_| {
                writes += 1;
                true
            });
            writes
        });

        sender
            .send(Message::Dirty(Kind::History, Instant::now()))
            .unwrap();
        sender.send(Message::Flush(done)).unwrap();
        wait.recv().unwrap();
        sender.send(Message::Stop).unwrap();
        assert_eq!(worker.join().unwrap(), 1);
    }
}
