/*!
My custom smart threads. Also reexports `std::thread::*` for convenience.

A smart thread is similiar to a normal thread, except it knows when
it is finished executing. At any time, a smart thread's handle may
poll its status.

# Types and functions added by this module
* [`spawn_smart`](fn.spawn_smart.html) Spawns a smart thread
* [`SmartHandle`](struct.SmartHandle.html) A handle to a smart thread
* [`ThreadStatus`](enum.ThreadStatus.html) The status of a smart thread

# Example
```
use kai::thread;

// Spawn a new smart thread that sleeps for a short amount of time
let handle = thread::spawn_smart(|| thread::sleep(std::time::Duration::from_millis(10)));
assert!(handle.status().is_running());
// Wait for the thread to finish
thread::sleep(std::time::Duration::from_millis(20));
assert!(handle.status().finished());

// Spawn a new smart thread that panics
let handle = thread::spawn_smart(|| panic!());
// Give the thread time to panic
thread::sleep(std::time::Duration::from_millis(10));
assert!(handle.status().panicked());
```
*/

use std::sync::{Arc, Mutex};

pub use std::thread::*;

/// The execution status of a thread
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadStatus {
    /// The thread is running
    Running,
    /// The thread finished running
    Finished,
    /// The thread panicked
    Panicked,
}

impl ThreadStatus {
    /// Check if the status indicated the thread is running
    pub fn is_running(self) -> bool {
        self == ThreadStatus::Running
    }
    /// Check if the status indicates the thread finished successfully
    pub fn finished(self) -> bool {
        self == ThreadStatus::Finished
    }
    /// Check if the status indicates the thread panicked
    pub fn panicked(self) -> bool {
        self == ThreadStatus::Panicked
    }
}

/// A thread handle that knows when the thread
/// is finished executing
pub struct SmartHandle<T> {
    handle: JoinHandle<Result<T>>,
    status: Arc<Mutex<ThreadStatus>>,
}

impl<T> SmartHandle<T> {
    /// Join the thread.
    /// Analgous to [`JoinHandle::join`](struct.JoinHandle.html#method.join)
    pub fn join(self) -> Result<T> {
        self.handle.join().and_then(|r| r)
    }
    /// Check if the thread is finished executing
    pub fn status(&self) -> ThreadStatus {
        self.status
            .lock()
            .map(|guard| ThreadStatus::clone(&*guard))
            .unwrap_or(ThreadStatus::Panicked)
    }
    /// Extracts a handle to the underlying thread
    pub fn thread(&self) -> &Thread {
        self.handle.thread()
    }
    /// Consume this handle and get the inner [`JoinHandle`](struct.JoinHandle.html)
    pub fn into_inner(self) -> JoinHandle<Result<T>> {
        self.handle
    }
}

/// Spawn a smart thread that knows when it is finished
pub fn spawn_smart<F, T>(f: F) -> SmartHandle<T>
where
    F: FnOnce() -> T + Send + std::panic::UnwindSafe + 'static,
    T: Send + 'static,
{
    let status = Arc::new(Mutex::new(ThreadStatus::Running));
    let status_clone = Arc::clone(&status);
    SmartHandle {
        status,
        handle: spawn(move || {
            let res = match std::panic::catch_unwind(f) {
                Ok(res) => res,
                Err(e) => {
                    if let Ok(mut status) = status_clone.lock() {
                        *status = ThreadStatus::Panicked;
                    }
                    return Err(e);
                }
            };
            if let Ok(mut status) = status_clone.lock() {
                *status = ThreadStatus::Finished;
            }
            Ok(res)
        }),
    }
}
