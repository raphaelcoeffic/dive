use std::sync::OnceLock;

use rustix::{
    fd::OwnedFd,
    fs::{openat, Dir, Mode, OFlags, CWD},
    io,
};

/// Handle on the `/proc` seen when first checking the thread count.
///
/// `/proc` may not resolve the calling process anymore after entering a
/// container's mount namespace or a chroot. Looking up `self` through the
/// original procfs still does, also from children forked into nested PID
/// namespaces, as procfs resolves `self` in its own PID namespace.
static PROC: OnceLock<io::Result<OwnedFd>> = OnceLock::new();

fn count_threads() -> io::Result<usize> {
    let proc = PROC
        .get_or_init(|| {
            openat(
                CWD,
                "/proc",
                OFlags::PATH | OFlags::DIRECTORY | OFlags::CLOEXEC,
                Mode::empty(),
            )
        })
        .as_ref()
        .map_err(|err| *err)?;

    let task = openat(
        proc,
        "self/task",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
        Mode::empty(),
    )?;

    let mut count = 0;
    for entry in Dir::read_from(&task)? {
        let name = entry?.file_name().to_bytes().to_owned();
        if name != b"." && name != b".." {
            count += 1;
        }
    }
    Ok(count)
}

/// Assert that the process is single-threaded.
///
/// Forking a multi-threaded process is unsafe: only the calling thread
/// survives in the child, but locks and other state held by other threads
/// remain "locked forever", and only async-signal-safe functions are legal
/// in the child until `exec`. dive's fork sites do non-trivial Rust work
/// in the child, so they require the process to be single-threaded.
///
/// Counts entries under `/proc/self/task`. The first call must happen
/// before entering other namespaces (see [`PROC`]). In debug builds a
/// violation panics; in release builds it logs a warning and proceeds.
pub fn assert_single_threaded() {
    let count = match count_threads() {
        Ok(count) => count,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_threads() {
        let before = count_threads().unwrap();
        let (tx, rx) = std::sync::mpsc::channel::<()>();
        let thread = std::thread::spawn(move || rx.recv());
        assert_eq!(count_threads().unwrap(), before + 1);
        drop(tx);
        thread.join().unwrap().unwrap_err();
    }
}
