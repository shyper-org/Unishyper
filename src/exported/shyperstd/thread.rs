use alloc::string::String;
use alloc::sync::Arc;
use alloc::boxed::Box;
use core::cell::UnsafeCell;

use super::io;
use crate::libs::thread as imp;

#[derive(Debug)]
pub struct Builder {
    // A name for the thread-to-be, for identification in panic messages
    name: Option<String>,
    // The size of the stack for the spawned thread in bytes
    stack_size: Option<usize>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            name: None,
            stack_size: None,
        }
    }

    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = Some(size);
        self
    }

    pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        unsafe { self.spawn_unchecked(f) }
    }

    pub unsafe fn spawn_unchecked<'a, F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let Builder { name, stack_size } = self;

        let stack_size = stack_size.unwrap_or(crate::arch::STACK_SIZE);

        let my_packet = Arc::new(Packet {
            result: UnsafeCell::new(None),
        });

        // let my_packet: Arc<UnsafeCell<Option<io::Result<T>>>> = Arc::new(UnsafeCell::new(None));
        let their_packet = my_packet.clone();

        let main = move || {
            let try_result = f();

            // SAFETY: `their_packet` as been built just above and moved by the
            // closure (it is an Arc<...>) and `my_packet` will be stored in the
            // same `JoinInner` as this closure meaning the mutation will be
            // safe (not modify it and affect a value far away).
            // unsafe { *their_packet.get() = Some(try_result) };
            // drop(their_packet);
            unsafe { *their_packet.result.get() = Some(try_result) };
            drop(their_packet);
        };

        Ok(JoinHandle(JoinInner {
            // SAFETY:
            //
            // `imp::Thread::new` takes a closure with a `'static` lifetime, since it's passed
            // through FFI or otherwise used with low-level threading primitives that have no
            // notion of or way to enforce lifetimes.
            //
            // As mentioned in the `Safety` section of this function's documentation, the caller of
            // this function needs to guarantee that the passed-in lifetime is sufficiently long
            // for the lifetime of the thread.
            //
            // Similarly, the `sys` implementation must guarantee that no references to the closure
            // exist after the thread has terminated, which is signaled by `Thread::join`
            // returning.
            native: Some(imp::spawn_raw(Box::new(main), name, stack_size)),
            packet: my_packet,
        }))
    }
}

/// Spawns a new thread, returning a [`JoinHandle`] for it.
pub fn spawn<F, T>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + 'static,
{
    Builder::new().spawn(f).expect("failed to spawn thread")
}

/// Gets a handle to the thread that invokes it.
///
/// # Examples
///
/// Getting a handle to the current thread with `thread::current()`:
pub fn current() -> imp::Thread {
    crate::libs::thread::current_thread().expect("current thread not exist, something went wrong!")
}

/// Cooperatively gives up a timeslice to the OS scheduler.
///
/// This calls the underlying OS scheduler's yield primitive, signaling
/// that the calling thread is willing to give up its remaining timeslice
/// so that the OS may schedule other threads on the CPU.
///
/// A drawback of yielding in a loop is that if the OS does not have any
/// other ready threads to run on the current CPU, the thread will effectively
/// busy-wait, which wastes CPU time and energy.
pub fn yield_now() {
    crate::libs::thread::thread_yield();
}

/// Puts the current thread to sleep for at least the specified amount of time.
///
/// The thread may sleep longer than the duration specified due to scheduling
/// specifics or platform-dependent functionality. It will never sleep less.
///
/// This function is blocking, and should not be used in `async` functions.
pub fn sleep(dur: core::time::Duration) {
    // imp::Thread::sleep(dur)
    crate::libs::thread::thread_block_current_with_timeout_us(dur.as_micros() as usize)
}

pub type ThreadId = crate::libs::thread::Tid;

////////////////////////////////////////////////////////////////////////////////
// JoinHandle
////////////////////////////////////////////////////////////////////////////////

// pub type Result<T> = core::result::Result<T, Box<dyn Any + Send + 'static>>;

struct Packet<T> {
    result: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for Packet<T> {}

/// Inner representation for JoinHandle
struct JoinInner<T> {
    native: Option<imp::Thread>,
    packet: Arc<Packet<T>>,
}

impl<T> JoinInner<T> {
    fn join(&mut self) -> io::Result<T> {
        self.native.take().unwrap().join();
        Arc::get_mut(&mut self.packet)
            .unwrap_or(&mut Packet {
                result: UnsafeCell::new(None),
            })
            .result
            .get_mut()
            .take()
            .ok_or_else(|| "bad state")
        // unsafe { (*self.packet.0.get()).take().unwrap() }
    }
}

/// An owned permission to join on a thread (block on its termination).
///
/// A `JoinHandle` *detaches* the associated thread when it is dropped, which
/// means that there is no longer any handle to the thread and no way to `join`
/// on it.
///
/// Due to platform restrictions, it is not possible to [`Clone`] this
/// handle: the ability to join a thread is a uniquely-owned permission.
///
/// This `struct` is created by the [`thread::spawn`] function and the
/// [`thread::Builder::spawn`] method.
pub struct JoinHandle<T>(JoinInner<T>);

unsafe impl<T> Send for JoinHandle<T> {}

unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    /// Extracts a handle to the underlying thread.
    pub fn thread(&self) -> imp::Thread {
        self.0.native.clone().unwrap()
    }

    pub fn join(mut self) -> io::Result<T> {
        self.0.join()
    }

    /// Checks if the associated thread has finished running its main function.
    ///
    /// `is_finished` supports implementing a non-blocking join operation, by checking
    /// `is_finished`, and calling `join` if it returns `true`. This function does not block. To
    /// block while waiting on the thread to finish, use [`join`][Self::join].
    ///
    /// This might return `true` for a brief moment after the thread's main
    /// function has returned, but before the thread itself has stopped running.
    /// However, once this returns `true`, [`join`][Self::join] can be expected
    /// to return quickly, without blocking for any significant amount of time.
    pub fn is_finished(&self) -> bool {
        Arc::strong_count(&self.0.packet) == 1
    }
}
