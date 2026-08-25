use std::sync::Mutex;
use tauri::async_runtime::JoinHandle;

/// Holds the in-flight OAuth task so the wizard can cancel it.
///
/// Only one flow can run at a time: the redirect listener binds a fixed port.
#[derive(Default)]
pub struct OauthState {
    task: Mutex<Option<JoinHandle<()>>>,
}

impl OauthState {
    /// Store a new task, aborting whatever was running before.
    pub fn replace(&self, handle: JoinHandle<()>) {
        let mut task = self.task.lock().unwrap();

        if let Some(previous) = task.take() {
            previous.abort();
        }

        *task = Some(handle);
    }

    pub fn cancel(&self) {
        let mut task = self.task.lock().unwrap();

        if let Some(handle) = task.take() {
            handle.abort();
        }
    }
}
