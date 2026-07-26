/// Autosave queue and trigger hooks for the puzzle game.
///
/// # Design
///
/// The autosave queue decouples trigger events (answer submission, hint reveal)
/// from the actual file I/O using a bounded `mpsc::sync_channel`.  A dedicated
/// worker thread drains the queue serially, ensuring:
///
/// - No save event is silently dropped (Req 3.5).
/// - The TUI thread is never blocked by disk I/O.
/// - Snapshots are captured at trigger time, so in-flight mutations do not
///   affect what gets written (Req 3.4).
///
/// For hint reveals (Req 3.2 — save *before* hint is displayed) callers use
/// `autosave_on_hint_reveal`, which performs a **synchronous** save on the
/// calling thread before returning the hint content.
#[cfg(not(target_arch = "wasm32"))]
pub mod queue {
    use std::path::{Path, PathBuf};
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::time::Instant;

    use crate::error::SaveError;
    use crate::models::Session;
    use crate::ops::save_session;

    // -----------------------------------------------------------------------
    // Types
    // -----------------------------------------------------------------------

    /// An event sent to the autosave worker thread.
    pub enum AutosaveEvent {
        /// Snapshot of the session to persist.
        Save(Session),
        /// Sentinel — shuts down the worker thread gracefully.
        Shutdown,
    }

    /// A warning produced when an autosave operation fails.
    ///
    /// The TUI should display this as a non-blocking overlay for ≤5 seconds.
    #[derive(Debug)]
    pub struct AutosaveWarning {
        /// Human-readable description of the failure (from `SaveError::to_string()`).
        pub message: String,
        /// Instant at which the failure was detected (use for 5 s auto-dismiss).
        pub triggered_at: Instant,
    }

    /// Handle to the autosave background worker.
    ///
    /// Drop to send a `Shutdown` sentinel and join the worker thread.
    pub struct AutosaveQueue {
        tx: SyncSender<AutosaveEvent>,
        worker: Option<std::thread::JoinHandle<()>>,
    }

    impl AutosaveQueue {
        /// Spawn the autosave worker.
        ///
        /// `save_dir` — directory where session files will be written as
        ///              `<session_id>.json`.
        ///
        /// `warn_tx`  — channel through which the worker reports `AutosaveWarning`
        ///              events back to the TUI.
        pub fn new(
            save_dir: PathBuf,
            warn_tx: mpsc::Sender<AutosaveWarning>,
        ) -> Self {
            // Bounded channel (capacity 64) prevents unbounded memory growth
            // while ensuring no event is dropped — the sender blocks briefly
            // when the queue is full rather than discarding.
            let (tx, rx) = mpsc::sync_channel::<AutosaveEvent>(64);
            let worker = std::thread::spawn(move || {
                run_worker(rx, save_dir, warn_tx);
            });
            AutosaveQueue {
                tx,
                worker: Some(worker),
            }
        }

        /// Send a session snapshot to the worker for asynchronous saving.
        ///
        /// Cloning the session here (not at the call site) keeps the public API
        /// simple and guarantees the snapshot is immutable from this point.
        pub fn enqueue(&self, session: &Session) {
            // `send` blocks only if the channel is at capacity (64 items),
            // which should never occur in normal gameplay.  We tolerate a send
            // error only if the worker thread has already exited.
            let _ = self.tx.send(AutosaveEvent::Save(session.clone()));
        }
    }

    impl Drop for AutosaveQueue {
        fn drop(&mut self) {
            // Signal shutdown and join.
            let _ = self.tx.send(AutosaveEvent::Shutdown);
            if let Some(handle) = self.worker.take() {
                let _ = handle.join();
            }
        }
    }

    fn run_worker(
        rx: Receiver<AutosaveEvent>,
        save_dir: PathBuf,
        warn_tx: mpsc::Sender<AutosaveWarning>,
    ) {
        for event in rx {
            match event {
                AutosaveEvent::Save(session) => {
                    let path = save_dir.join(format!("{}.json", session.session_id.as_str()));
                    if let Err(e) = save_session(&session, &path) {
                        let warning = AutosaveWarning {
                            message: e.to_string(),
                            triggered_at: Instant::now(),
                        };
                        // Ignore send error — TUI may have exited.
                        let _ = warn_tx.send(warning);
                    }
                }
                AutosaveEvent::Shutdown => break,
            }
        }
    }

    // -----------------------------------------------------------------------
    // Trigger hooks
    // -----------------------------------------------------------------------

    /// Trigger an autosave when a player submits an answer.
    ///
    /// Enqueues a snapshot of `session` in the background worker.  The save
    /// completes within 2 seconds in normal conditions (Req 3.1).
    pub fn autosave_on_answer_submit(queue: &AutosaveQueue, session: &Session) {
        queue.enqueue(session);
    }

    /// Trigger a synchronous save when a player reveals a hint.
    ///
    /// The save completes **before** this function returns, guaranteeing that
    /// the save precedes any display of hint content (Req 3.2).
    ///
    /// Returns `Err(SaveError)` if the save fails — callers should surface
    /// this to the player and may choose to still display the hint.
    pub fn autosave_on_hint_reveal(session: &Session, save_dir: &Path) -> Result<(), SaveError> {
        let path = save_dir.join(format!("{}.json", session.session_id.as_str()));
        save_session(session, &path)
    }
}
