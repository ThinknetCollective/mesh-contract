/// CLI `--resume <session_id>` flag support.
///
/// This module provides the logic needed to integrate session resumption into
/// a command-line interface.  It is intentionally thin: callers parse their
/// own `Args` struct (e.g. via `clap`) and then call `handle_resume` to
/// perform the load and return the restored session.
///
/// # Example integration with `clap`
///
/// ```ignore
/// #[derive(clap::Parser)]
/// struct Args {
///     /// Resume a previously saved session.
///     #[arg(long, value_name = "session_id")]
///     resume: Option<String>,
/// }
///
/// fn main() {
///     let args = Args::parse();
///     if let Some(ref id) = args.resume {
///         match puzzle_persistence::cli::handle_resume(id, &save_dir) {
///             Ok(session) => launch_tui(session),
///             Err(e) => {
///                 puzzle_persistence::cli::report_error_and_exit(&e, id);
///             }
///         }
///     }
/// }
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub mod resume {
    use std::path::Path;

    use crate::error::SaveError;
    use crate::models::Session;
    use crate::ops::load_session;

    /// Validate `session_id` and load the corresponding save file from `save_dir`.
    ///
    /// # Errors
    ///
    /// - `SaveError::Corrupted` — `session_id` is empty, too long (>36 chars),
    ///   or contains invalid characters; or the file has structural issues.
    /// - `SaveError::VersionMismatch` — the save file schema version differs
    ///   from the current binary's `SAVE_FORMAT_VERSION`.
    /// - `SaveError::Io` — the file does not exist or cannot be read.
    pub fn handle_resume(session_id: &str, save_dir: &Path) -> Result<Session, SaveError> {
        validate_resume_id(session_id)?;
        let path = save_dir.join(format!("{}.json", session_id));
        load_session(&path).map_err(|e| annotate_error(e, session_id, &path))
    }

    /// Validate `session_id` as supplied to `--resume`.
    ///
    /// Requirements: 1–36 printable non-whitespace ASCII characters.
    /// Returns `SaveError::Corrupted` with a usage message on violation.
    pub fn validate_resume_id(session_id: &str) -> Result<(), SaveError> {
        if session_id.is_empty()
            || session_id.len() > 36
            || session_id
                .chars()
                .any(|c| c.is_whitespace() || !c.is_ascii_graphic())
        {
            return Err(SaveError::Corrupted {
                message: format!(
                    "invalid --resume value '{}': must be 1–36 printable non-whitespace characters\nusage: --resume <session_id>",
                    session_id
                ),
            });
        }
        Ok(())
    }

    /// Format a `SaveError` into a human-readable message for stderr that
    /// always includes the `session_id` and the nature of the error.
    pub fn format_resume_error(err: &SaveError, session_id: &str) -> String {
        match err {
            SaveError::Io(io_err)
                if io_err.kind() == std::io::ErrorKind::NotFound =>
            {
                format!(
                    "error: no saved session found for id '{}'\n       (looked for {}.json)\nhint:  start a new game or check the session id",
                    session_id, session_id
                )
            }
            SaveError::VersionMismatch { found, expected } => {
                format!(
                    "error: save file for session '{}' was created by schema v{} but this binary expects v{}\nhint:  delete the save file and start a new session",
                    session_id, found, expected
                )
            }
            SaveError::Corrupted { message } => {
                format!(
                    "error: save file for session '{}' is corrupted: {}\nhint:  delete the save file and start a new session",
                    session_id, message
                )
            }
            other => {
                format!("error: could not load session '{}': {}", session_id, other)
            }
        }
    }

    /// Print an error message to stderr and exit with code 1.
    ///
    /// Called by the CLI entry point when `handle_resume` returns an `Err`.
    /// This function never returns (`-> !`).
    pub fn report_error_and_exit(err: &SaveError, session_id: &str) -> ! {
        eprintln!("{}", format_resume_error(err, session_id));
        std::process::exit(1);
    }

    // Annotate an error with the session_id and path for richer CLI messages.
    fn annotate_error(err: SaveError, session_id: &str, path: &std::path::Path) -> SaveError {
        match err {
            // Re-wrap I/O errors with path context.
            SaveError::Io(io_err) => SaveError::Io(std::io::Error::new(
                io_err.kind(),
                format!(
                    "session '{}' ({}): {}",
                    session_id,
                    path.display(),
                    io_err
                ),
            )),
            // VersionMismatch and Corrupted already contain enough context.
            other => other,
        }
    }
}
