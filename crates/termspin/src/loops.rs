use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    util::{DisplayFn, SHARED_LOCK},
    Frames,
};

/// Run the loop with the given callback.
///
/// # Example
///
/// ```no_run
/// # use termspin::{Loop, Line};
/// # use termspin::spinner;
/// # use std::time::Duration;
///
/// let l = Loop::new(
///     Duration::from_millis(100),
///     Line::new(spinner::from_iter([r"\", "|", "/"]))
/// );
///
/// // Run the loop while blocking the current thread.
/// l.run(|out| print!("{out}"));
/// ```
#[derive(Debug)]
pub struct Loop<F: Frames> {
    inner: Arc<Mutex<LoopInner<F>>>,
}

impl<F: Frames> Clone for Loop<F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[allow(clippy::missing_panics_doc)]
impl<F: Frames> Loop<F> {
    /// Create a new loop that updates at the given
    /// interval.
    pub fn new(interval: Duration, frames: F) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LoopInner {
                running: false,
                stop: false,
                auto_stop: true,
                reset: false,
                delay: interval,
                wait: None,
                frames,
            })),
        }
    }

    /// Run the loop with the given callback.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use termspin::{Loop, Line};
    /// # use termspin::spinner;
    /// # use std::time::Duration;
    ///
    /// let l = Loop::new(
    ///     Duration::from_millis(100),
    ///     Line::new(spinner::from_iter([r"\", "|", "/"]))
    /// );
    ///
    /// // Run the loop while blocking the current thread.
    /// l.run(|out| print!("{out}"));
    /// ```
    #[allow(clippy::missing_errors_doc)]
    pub fn run(
        &self,
        mut f: impl FnMut(&dyn core::fmt::Display) -> io::Result<()>,
    ) -> io::Result<()> {
        let mut first = true;
        let mut shared_lock = None;
        self.inner.lock().unwrap().stop = false;
        self.inner.lock().unwrap().running = true;
        loop {
            let mut inner = self.inner.lock().unwrap();

            if (inner.auto_stop && Arc::strong_count(&self.inner) == 1) || inner.stop {
                break;
            }

            if let Some(wait) = inner.wait.take() {
                thread::sleep(wait);
            }

            if inner.reset {
                inner.reset = false;
                inner.frames.reset();
            } else if !first {
                f(&DisplayFn::new(|f| inner.frames.clear(f)))?;
            }

            first = false;

            drop(shared_lock.take());
            // Allow other threads to take the lock.
            thread::sleep(Duration::from_micros(1));
            shared_lock = Some(SHARED_LOCK.lock().unwrap());

            f(&inner.frames)?;

            inner.frames.advance();
            let delay = inner.delay;
            drop(inner);

            thread::sleep(delay);
        }
        self.inner.lock().unwrap().running = false;

        Ok(())
    }

    /// Run the loop outputting frames to the given stream.
    #[allow(clippy::missing_errors_doc)]
    pub fn run_stream(&self, mut stream: impl std::io::Write) -> io::Result<()> {
        self.run(|f| {
            write!(stream, "{}", f)?;
            stream.flush()
        })
    }

    /// A convenience function to clear the given stream.
    #[allow(clippy::missing_errors_doc)]
    pub fn clear_stream(&self, mut stream: impl std::io::Write) -> io::Result<()> {
        write!(
            stream,
            "{}",
            DisplayFn::new(|f| self.inner.lock().unwrap().frames.clear(f))
        )
    }

    /// Spawn the loop on a separate thread,
    /// no-op if the loop is already running.
    pub fn spawn_stream<S>(&self, stream: S)
    where
        S: std::io::Write + Send + 'static,
    {
        if self.inner.lock().unwrap().running {
            return;
        }

        let this = self.clone();

        thread::spawn(move || {
            this.run_stream(stream).unwrap();
        });
    }

    /// Stop a running loop.
    pub fn stop(&self) {
        self.inner.lock().unwrap().stop = true;
    }

    /// Wait for the given duration before the
    /// next cycle.
    pub fn wait(&self, duration: Duration) {
        self.inner.lock().unwrap().wait = Some(duration);
    }

    /// Reset the frames of the loop.
    pub fn reset(&self) {
        self.inner.lock().unwrap().reset = true;
    }

    /// Clone the inner frames.
    #[must_use]
    pub fn inner(&self) -> F
    where
        F: Clone,
    {
        self.inner.lock().unwrap().frames.clone()
    }

    /// Exit the running loop if only one instance
    /// of the loop exists, defaults to `true`.
    ///
    /// It is useful when you spawn the spawn the loop
    /// on a separate thread that should exit when
    /// all handles to it go out of scope.
    pub fn auto_stop(&self, stop: bool) {
        self.inner.lock().unwrap().auto_stop = stop;
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
struct LoopInner<F>
where
    F: Frames,
{
    running: bool,
    stop: bool,
    auto_stop: bool,
    reset: bool,
    delay: Duration,
    wait: Option<Duration>,
    frames: F,
}
