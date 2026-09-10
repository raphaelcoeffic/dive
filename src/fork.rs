use std::fs;

/// Assert that the process is single-threaded.
///
/// Forking a multi-threaded process is unsafe: only the calling thread
/// survives in the child, but locks and other state held by other threads
/// remain "locked forever", and only async-signal-safe functions are legal
/// in the child until `exec`. dive's fork sites do non-trivial Rust work
/// in the child, so they require the process to be single-threaded.
///
/// Counts entries under `/proc/self/task`. In debug builds a violation
/// panics; in release builds it logs a warning and proceeds.
pub fn assert_single_threaded() {
    let count = match fs::read_dir("/proc/self/task") {
        Ok(entries) => entries.count(),
        Err(err) => {
            log::warn!("could not read /proc/self/task: {err}");
            return;
        }
    };
    if count > 1 {
        log::warn!(
            "fork() called with {count} threads alive; \
             child state may be inconsistent"
        );
        debug_assert!(
            count == 1,
            "fork() requires a single-threaded process, found {count} threads"
        );
    }
}
