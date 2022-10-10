//! Various spinner implementations and utilities for [`Frames`].

use crate::Frames;

/// Create frames from an iterator.
///
/// # Example
///
/// ```
/// # use termspin::spinner::from_iter;
/// from_iter([r"\", "|", "/"]);
/// ```
pub fn from_iter<I, F>(iter: I) -> FromIter<I::IntoIter, F>
where
    I: IntoIterator<Item = F>,
    I::IntoIter: Clone,
    F: core::fmt::Display,
{
    FromIter::new(iter.into_iter())
}

/// Create empty spinner that does not display anything.
#[must_use]
pub fn empty() -> Empty {
    Empty
}

/// Create a spinner from commonly used dots.
#[must_use]
pub const fn dots() -> FromArray<10, &'static str> {
    FromArray::new(["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
}

/// Create a spinner from an array.
pub const fn from_array<const N: usize, F>(array: [F; N]) -> FromArray<N, F>
where
    F: core::fmt::Display,
{
    FromArray::new(array)
}

/// Frames returned by [`from_iter`].
#[derive(Debug, Clone, Copy)]
pub struct FromIter<I, F>
where
    I: Iterator<Item = F> + Clone,
    F: core::fmt::Display,
{
    start: I,
    current: I,
    frame: Option<F>,
}

impl<I, F> FromIter<I, F>
where
    I: Iterator<Item = F> + Clone,
    F: core::fmt::Display,
{
    /// Create frames from an iterator.
    pub fn new(iter: I) -> Self {
        let frame = iter.clone().next();

        Self {
            start: iter.clone(),
            current: iter,
            frame,
        }
    }
}

impl<I, F> core::fmt::Display for FromIter<I, F>
where
    I: Iterator<Item = F> + Clone,
    F: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(frame) = &self.frame {
            frame.fmt(f)?;
        }
        Ok(())
    }
}

impl<I, F> Frames for FromIter<I, F>
where
    I: Iterator<Item = F> + Clone + Send + Sync + 'static,
    F: core::fmt::Display + Send + Sync + 'static,
{
    fn advance(&mut self) {
        if let Some(f) = self.current.next() {
            self.frame = Some(f);
        } else {
            self.current = self.start.clone();
            self.frame = self.current.next();
        }
    }

    fn reset(&mut self) {
        self.frame = self.start.clone().next();
        self.current = self.start.clone();
    }
}

/// Empty frames that do not display anything.
pub struct Empty;

impl core::fmt::Display for Empty {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl Frames for Empty {
    fn advance(&mut self) {}
    fn print_len(&self) -> Option<usize> {
        Some(0)
    }
}

/// Frames returned by [`from_array`].
#[derive(Clone, Copy)]
pub struct FromArray<const N: usize, F>
where
    F: core::fmt::Display,
{
    idx: usize,
    array: [F; N],
}

impl<const N: usize, F> FromArray<N, F>
where
    F: core::fmt::Display,
{
    /// # Panics
    ///
    /// Panic if the array is empty.
    #[must_use]
    pub const fn new(array: [F; N]) -> Self {
        assert!(N != 0, "the array cannot be empty.");
        Self { idx: 0, array }
    }
}

impl<const N: usize, F> core::fmt::Display for FromArray<N, F>
where
    F: core::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.array[self.idx].fmt(f)
    }
}

impl<const N: usize, F> Frames for FromArray<N, F>
where
    F: core::fmt::Display + Send + Sync + 'static,
{
    fn advance(&mut self) {
        if self.idx == self.array.len() - 1 {
            self.idx = 0;
        } else {
            self.idx += 1;
        }
    }

    fn reset(&mut self) {
        self.idx = 0;
    }
}
