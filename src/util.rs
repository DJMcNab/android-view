use std::{
    fs::File,
    io::{BufRead, BufReader},
    os::fd::{FromRawFd, RawFd},
};

use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean};

pub(crate) fn as_jboolean(flag: bool) -> jboolean {
    if flag { JNI_TRUE } else { JNI_FALSE }
}

pub fn forward_stdio_to_logcat() -> std::thread::JoinHandle<std::io::Result<()>> {
    // XXX: make this stdout/stderr redirection an optional / opt-in feature?...

    // Safety: Trivial from libc function usage
    let file = unsafe {
        let mut logpipe: [RawFd; 2] = Default::default();
        libc::pipe2(logpipe.as_mut_ptr(), libc::O_CLOEXEC);
        libc::dup2(logpipe[1], libc::STDOUT_FILENO);
        libc::dup2(logpipe[1], libc::STDERR_FILENO);
        libc::close(logpipe[1]);

        File::from_raw_fd(logpipe[0])
    };

    std::thread::Builder::new()
        .name("stdio-to-logcat".to_string())
        .spawn(move || {
            let mut reader = BufReader::new(file);
            let mut buffer = String::new();
            loop {
                buffer.clear();
                let len = match reader.read_line(&mut buffer) {
                    Ok(len) => len,
                    Err(e) => {
                        log::error!("Logcat forwarder failed to read stdin/stderr: {e:?}");
                        break Err(e);
                    }
                };
                if len == 0 {
                    break Ok(());
                } else {
                    log::info!(target: "VelloStdoutStderr", "{buffer}");
                }
            }
        })
        .expect("Failed to start stdout/stderr to logcat forwarder thread")
}

fn log_panic(panic: Box<dyn std::any::Any + Send>) {
    if let Some(panic) = panic.downcast_ref::<String>() {
        log::error!(target: "RustPanic", "{panic}");
    } else if let Some(panic) = panic.downcast_ref::<&str>() {
        log::error!(target: "RustPanic", "{panic}");
    } else {
        log::error!(target: "UnknownPanic", "Got panic of unknown type at: {:x?}", std::ptr::from_ref(&*panic));
    }
}

/// Run a closure and abort the program if it panics.
///
/// This is generally used to ensure Rust callbacks won't unwind past the JNI boundary, which leads
/// to undefined behaviour.
///
/// TODO(rib): throw a Java exception instead of aborting. An Android Activity does not necessarily
/// own the entire process because other application Services (or even Activities) may run in
/// threads within the same process, and so we're tearing down too much by aborting the process.
pub fn abort_on_panic<R>(f: impl FnOnce() -> R) -> R {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|panic| {
        // Try logging the panic before aborting
        //
        // Just in case our attempt to log a panic could itself cause a panic we use a
        // second catch_unwind here.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| log_panic(panic)));
        std::process::abort();
    })
}
