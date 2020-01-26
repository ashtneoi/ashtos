use core::ops::{Deref, DerefMut, Drop};
use core::marker::PhantomData;
use core::sync::atomic::{
    AtomicBool,
    Ordering,
};

pub(crate) struct Mutex<T> {
    inner: T,
    locked: AtomicBool,
}

unsafe impl<T> Sync for Mutex<T> { }

impl<T> Mutex<T> {
    pub const fn new(t: T) -> Self {
        Self { inner: t, locked: AtomicBool::new(false) }
    }

    fn try_lock(&self) -> bool {
        // TODO: SeqCst is very strict. Can we loosen it?
        let already_locked = self.locked.compare_and_swap(
            false,
            true,
            Ordering::SeqCst,
        );
        !already_locked
    }

    fn lock_spin(&self) {
        while !self.try_lock() { }
    }

    fn unlock(&self) {
        // TODO: SeqCst is very strict. Can we loosen it?
        self.locked.store(false, Ordering::SeqCst);
    }

    pub fn with_lock<'a>(&'a self) -> MutexGuard<'a, T> {
        self.lock();
        MutexGuard {
            inner: &self.inner as *const T as *mut T,
            mutex: self,
        }
    }
}

pub(crate) struct MutexGuard<'a, T> {
    inner: *mut T,
    mutex: &'a Mutex<T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}
