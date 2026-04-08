use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

static APP_HANDLE: OnceLock<RwLock<Option<AppHandle>>> = OnceLock::new();

pub fn init(app_handle: AppHandle) {
    let lock = APP_HANDLE.get_or_init(|| RwLock::new(None));
    // We can't await here, so use try_write
    if let Ok(mut guard) = lock.try_write() {
        *guard = Some(app_handle);
    }
}

fn now() -> String {
    chrono::Local::now().format("%H:%M:%S%.3f").to_string()
}

fn emit(level: LogLevel, target: &str, message: String) {
    let entry = LogEntry {
        timestamp: now(),
        level,
        target: target.to_string(),
        message,
    };

    let lock = match APP_HANDLE.get() {
        Some(l) => l,
        None => return,
    };
    if let Ok(guard) = lock.try_read() {
        if let Some(handle) = guard.as_ref() {
            let _ = handle.emit("log:entry", &entry);
        }
    }
}

pub fn info(target: &str, message: impl Into<String>) {
    emit(LogLevel::Info, target, message.into());
}

pub fn warn(target: &str, message: impl Into<String>) {
    emit(LogLevel::Warn, target, message.into());
}

pub fn error(target: &str, message: impl Into<String>) {
    emit(LogLevel::Error, target, message.into());
}

pub fn debug(target: &str, message: impl Into<String>) {
    emit(LogLevel::Debug, target, message.into());
}
