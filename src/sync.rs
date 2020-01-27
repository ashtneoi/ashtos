use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{
    AtomicBool,
    Ordering,
};

pub(crate) struct SpinLock<T> {
    inner: T,
    locked: AtomicBool,
}

unsafe impl<T> Sync for SpinLock<T> { }

impl<T> SpinLock<T> {
    pub const fn new(t: T) -> Self {
        Self { inner: t, locked: AtomicBool::new(false) }
    }

    fn try_lock(&self) -> bool {
        let already_locked = self.locked.compare_and_swap(
            false,
            true,
            Ordering::Acquire,
        );
        !already_locked
    }

    // TODO: Are `lock()` and `unlock()` unsafe?

    fn just_lock(&self) {
        while !self.try_lock() { }
    }

    fn just_unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }

    pub fn lock<'a>(&'a self) -> SpinLockGuard<'a, T> {
        self.just_lock();
        SpinLockGuard {
            inner: &self.inner as *const T as *mut T,
            lock: self,
        }
    }
}

pub(crate) struct SpinLockGuard<'a, T> {
    inner: *mut T,
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.just_unlock();
    }
}
