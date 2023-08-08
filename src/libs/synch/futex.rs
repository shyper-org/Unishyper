use core::sync::atomic::AtomicU32;
use core::sync::atomic::Ordering::SeqCst;
use alloc::collections::VecDeque;

use ahash::RandomState;
use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use bitflags::bitflags;

use crate::libs::synch::spinlock::SpinlockIrqSave;
use crate::libs::timer::current_us;
use crate::libs::thread::{
    Thread, current_thread, thread_yield, thread_block_current_with_timeout_us,
    thread_block_current, thread_wake,
};

static PARKING_LOT: SpinlockIrqSave<HashMap<usize, VecDeque<Thread>, RandomState>> =
    SpinlockIrqSave::new(HashMap::with_hasher(RandomState::with_seeds(0, 0, 0, 0)));

bitflags! {
    pub struct Flags: u32 {
        /// Use a relative timeout
        const RELATIVE = 0b01;
    }
}

/// If the value at address matches the expected value, park the current thread until it is either
/// woken up with `futex_wake` (returns 0) or the specified timeout elapses (returns -ETIMEDOUT).
///
/// The timeout is given in microseconds. If [`Flags::RELATIVE`] is given, it is interpreted as
/// relative to the current time. Otherwise it is understood to be an absolute time
/// (see `get_timer_ticks`).
pub fn futex_wait(address: &AtomicU32, expected: u32, timeout: Option<usize>, flags: Flags) -> i32 {
    let mut parking_lot = PARKING_LOT.lock();
    // Check the futex value after locking the parking lot so that all changes are observed.
    if address.load(SeqCst) != expected {
        return -1;
    }

    let wakeup_time = if flags.contains(Flags::RELATIVE) {
        timeout.and_then(|t| current_us().checked_add(t))
    } else {
        timeout
    };

    let timeout = if flags.contains(Flags::RELATIVE) {
        timeout
    } else {
        timeout.and_then(|t| t.checked_sub(current_us()))
    };

    match timeout {
        Some(time) => thread_block_current_with_timeout_us(time),
        None => thread_block_current(),
    };

    let current_thread = match current_thread() {
        Ok(t) => t,
        Err(e) => {
            warn!("no current thread");
            return -(e as i32);
        }
    };
    parking_lot
        .entry(address.as_ptr().addr())
        .or_default()
        .push_back(current_thread.clone());
    drop(parking_lot);

    loop {
        thread_yield();

        let mut parking_lot = PARKING_LOT.lock();
        if wakeup_time.is_some_and(|t| t <= current_us()) {
            let mut wakeup = true;
            // Timeout occurred, try to remove ourselves from the waiting queue.
            if let Entry::Occupied(mut queue) = parking_lot.entry(address.as_ptr().addr()) {
                // If we are not in the waking queue, this must have been a wakeup.
                let vec_queue = queue.get_mut();
                let mut i = 0;
                while i != vec_queue.len() {
                    if vec_queue[i].id() == current_thread.id() {
                        vec_queue.remove(i);
                        wakeup = false;
                    } else {
                        i += 1;
                    }
                }
                if queue.get().is_empty() {
                    queue.remove();
                }
            }

            if wakeup {
                return 0;
            } else {
                return -1;
            }
        } else {
            // If we are not in the waking queue, this must have been a wakeup.
            let wakeup = !parking_lot
                .get(&address.as_ptr().addr())
                .is_some_and(|queue| queue.contains(&current_thread));

            if wakeup {
                return 0;
            } else {
                // A spurious wakeup occurred, sleep again.
                // Tasks do not change core, so the handle in the parking lot is still current.
                match timeout {
                    Some(time) => thread_block_current_with_timeout_us(time),
                    None => thread_block_current(),
                };
            }
        }
        drop(parking_lot);
    }
}

/// Wake `count` threads waiting on the futex at address. Returns the number of threads
/// woken up (saturates to `i32::MAX`). If `count` is `i32::MAX`, wake up all matching
/// waiting threads. If `count` is negative, returns -EINVAL.
pub fn futex_wake(address: &AtomicU32, count: i32) -> i32 {
    if count < 0 {
        return -1;
    }

    let mut parking_lot = PARKING_LOT.lock();
    let mut queue = match parking_lot.entry(address.as_ptr().addr()) {
        Entry::Occupied(entry) => entry,
        Entry::Vacant(_) => return 0,
    };

    let mut woken = 0;
    while woken != count || count == i32::MAX {
        match queue.get_mut().pop_front() {
            Some(t) => thread_wake(&t),
            None => break,
        }
        woken = woken.saturating_add(1);
    }

    if queue.get().is_empty() {
        queue.remove();
    }

    woken
}
