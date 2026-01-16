use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: Guards need to contain the lock at their time of creation
        unsafe {
            self.mutex.lock.unlock();
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Guard is gaurenteed to have sole ownership
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: Guard is gaurenteed to have sole ownership
        unsafe { &mut *self.mutex.data.get() }
    }
}

#[derive(Debug)]
pub struct Mutex<T> {
    lock: Lock,
    data: UnsafeCell<T>,
}

// SAFETY: We manually implement thread safety with atomics
unsafe impl<T: Send> Send for Mutex<T> {}
// SAFETY: We manually implement thread safety with atomics
unsafe impl<T: Sync> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub fn new(data: T) -> Self {
        Self {
            lock: Lock::new(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.lock.lock();
        MutexGuard { mutex: self }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self.lock.try_lock() {
            Some(MutexGuard { mutex: self })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Lock {
    is_locked: AtomicBool,
}

impl Lock {
    pub const fn new() -> Self {
        Self {
            is_locked: AtomicBool::new(false),
        }
    }

    pub fn lock(&self) {
        while !self.try_lock() {
            core::hint::spin_loop();
        }
    }

    pub fn try_lock(&self) -> bool {
        self.is_locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Unlock
    ///
    /// # Safety
    ///
    /// Lock must currently be set to true and owned by the caller
    pub unsafe fn unlock(&self) {
        self.is_locked.store(false, Ordering::SeqCst);
    }
}

impl Default for Lock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use core::time::Duration;
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_lock() {
        // Starts unlocked
        let lock = Lock::new();
        assert!(!lock.is_locked.load(Ordering::Relaxed));

        // Now is locked
        lock.lock();
        assert!(lock.is_locked.load(Ordering::Relaxed));

        // Can't lock two times
        assert!(!lock.try_lock());

        // Now unlocked
        unsafe {
            lock.unlock();
        }
        assert!(!lock.is_locked.load(Ordering::Relaxed));

        // We should be able to lock again
        assert!(lock.try_lock());
        assert!(lock.is_locked.load(Ordering::Relaxed));
    }

    #[track_caller]
    fn take_and_test_mutex<T>(mutex: &Mutex<T>, target: T)
    where
        T: std::fmt::Debug + Eq + Clone,
    {
        let mut lock = mutex.lock();
        println!("setting {target:?}");
        assert!(mutex.lock.is_locked.load(Ordering::SeqCst));
        *lock = target.clone();
        assert_eq!(*lock.deref_mut(), target);
        assert_eq!(*lock.deref(), target);
    }

    #[test]
    fn test_mutex_good() {
        // Try lots of times to bring out possible hardware dependent bugs
        for _ in 0..100 {
            test_mutex_impl();
        }

        let mutex = Mutex::new(String::from("Hello"));

        let lock = mutex.lock();
        if lock.deref() == "Hello" {
            println!("works");
        } else {
            println!("doesn't work");
        }
    }

    fn test_mutex_impl() {
        let data = String::from("Hello");
        let mutex = Mutex::new(data);

        assert!(!mutex.lock.is_locked.load(Ordering::SeqCst));
        let mutex_ref = Arc::new(mutex);
        {
            let handles: Vec<_> = (0..100)
                .map(|ii| {
                    let mutex_ref_clone = mutex_ref.clone();
                    std::thread::spawn(move || {
                        let str = format!("{ii} says hello to {ii}");
                        let dur = Duration::from_millis(1);
                        std::thread::sleep(dur);
                        take_and_test_mutex(&mutex_ref_clone, str);
                    })
                })
                .collect();
            handles.into_iter().for_each(|h| h.join().unwrap());
        }
        assert!(!mutex_ref.lock.is_locked.load(Ordering::SeqCst));

        println!("end value {}", mutex_ref.lock().deref());
    }
}
