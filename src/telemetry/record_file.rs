//! Session files: when the game runs in a window, every finished wave is
//! appended as one JSON line to `playtests/session-<unix time>.jsonl`, so a
//! human's runs can sit in the same report as the bots. Headless apps (tests,
//! bots) have no window and write nothing.

use super::api::{SessionFileEnabled, SessionRecord, WaveRecord};
use super::session_line::WaveLine;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Folder (relative to where the game is started) that session files go to.
pub const SESSION_DIR: &str = "playtests";

#[derive(Resource, Debug, Default)]
pub(super) struct SessionFile {
    path: Option<PathBuf>,
    written: usize,
    failed: bool,
}

pub(super) fn write_session_file(
    window: Query<(), With<PrimaryWindow>>,
    enabled: Res<SessionFileEnabled>,
    session: Res<SessionRecord>,
    mut file: ResMut<SessionFile>,
) {
    if file.failed || !enabled.0 || window.is_empty() {
        return;
    }
    // A wave line goes out once the director's decision for it is known.
    let ready = session.waves[file.written..]
        .iter()
        .take_while(|w| w.decision.is_some())
        .count();
    if ready == 0 {
        return;
    }
    let path = file.path.get_or_insert_with(new_session_path).clone();
    let lines = &session.waves[file.written..file.written + ready];
    match append(&path, lines) {
        Ok(()) => file.written += ready,
        Err(e) => {
            warn!("session file {} not written: {e}", path.display());
            file.failed = true;
        }
    }
}

fn new_session_path() -> PathBuf {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    Path::new(SESSION_DIR).join(format!("session-{secs}.jsonl"))
}

fn append(path: &Path, waves: &[WaveRecord]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let mut out = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    for wave in waves {
        let line = serde_json::to_string(&WaveLine::from(wave)).map_err(|e| e.to_string())?;
        writeln!(out, "{line}").map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Read a session file written by the game back into a record.
pub fn read_session_file(path: &Path) -> Result<SessionRecord, String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut session = SessionRecord::default();
    for (n, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|e| format!("{}: {e}", path.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let parsed: WaveLine = serde_json::from_str(&line)
            .map_err(|e| format!("{}:{}: {e}", path.display(), n + 1))?;
        session.waves.push(parsed.into_record()?);
    }
    Ok(session)
}
