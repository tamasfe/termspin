use std::sync::{Arc, Mutex, MutexGuard};

use crate::Frames;

pub(crate) struct DisplayFn<F>
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> core::fmt::Result,
{
    f: F,
}

impl<F> DisplayFn<F>
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> core::fmt::Result,
{
    pub(crate) fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> core::fmt::Display for DisplayFn<F>
where
    F: Fn(&mut std::fmt::Formatter<'_>) -> core::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.f)(f)
    }
}

pub static SHARED_LOCK: Mutex<()> = Mutex::new(());

/// A convenience wrapper for `Arc<Mutex<_>>`
/// that implements [`Frames`].
#[must_use]
#[derive(Debug)]
pub struct SharedFrames<F>
where
    F: Frames,
{
    pub(crate) inner: Arc<Mutex<F>>,
}

impl<F> Eq for SharedFrames<F> where F: Frames {}

impl<F> PartialEq for SharedFrames<F>
where
    F: Frames,
{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<F> SharedFrames<F>
where
    F: Frames,
{
    /// Create a new shared value.
    pub fn new(frames: F) -> Self {
        Self {
            inner: Arc::new(Mutex::new(frames)),
        }
    }

    /// Lock this shared object and the global shared lock.
    /// 
    /// # Deadlocks
    /// 
    /// This function also locks a global lock that is
    /// used to uphold the guarantee that frames will not
    /// change between displaying and clearing (otherwise
    /// groups could clear more lines than they displayed).
    /// 
    /// This means that locking even two different `Shared`
    /// objects on the same thread will lead to a deadlock.
    #[allow(clippy::missing_panics_doc)]
    pub fn lock(&self) -> SharedLockGuard<F> {
        SharedLockGuard {
            _shared_lock: SHARED_LOCK.lock().unwrap(),
            inner_lock: self.inner.lock().unwrap(),
        }
    }
}

impl<F> Clone for SharedFrames<F>
where
    F: Frames,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<F> core::fmt::Display for SharedFrames<F>
where
    F: Frames,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.lock().unwrap().fmt(f)
    }
}

impl<F> Frames for SharedFrames<F>
where
    F: Frames,
{
    fn advance(&mut self) {
        self.inner.lock().unwrap().advance();
    }

    fn reset(&mut self) {
        self.inner.lock().unwrap().reset();
    }

    fn clear(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        self.inner.lock().unwrap().clear(f)
    }

    fn lines(&self) -> usize {
        self.inner.lock().unwrap().lines()
    }
}

/// A lock that includes the global shared lock.
#[must_use]
pub struct SharedLockGuard<'l, F> {
    _shared_lock: MutexGuard<'l, ()>,
    inner_lock: MutexGuard<'l, F>,
}

impl<'l, F> std::ops::Deref for SharedLockGuard<'l, F> {
    type Target = F;

    fn deref(&self) -> &Self::Target {
        &*self.inner_lock
    }
}
impl<'l, F> std::ops::DerefMut for SharedLockGuard<'l, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.inner_lock
    }
}
