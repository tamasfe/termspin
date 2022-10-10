//! A library for terminal multi-line spinners based purely on ANSI escape sequences.
//! 
//! # Example
#![warn(clippy::pedantic, missing_docs)]

use std::{
    borrow::Cow,
    fmt::{Display, Write},
};

use ansi::{ClearLine, CursorUp};
use downcast::AnySync;
use util::DisplayFn;

pub mod ansi;
mod loops;
pub mod spinner;
mod util;

pub use loops::Loop;
pub use util::SharedFrames;

/// Frames that can be printed to the terminal via
/// [`fmt::Display`](core::fmt::Display).
///
/// The printed text should not end with a new line.
pub trait Frames: AnySync + core::fmt::Display {
    /// Advance to the next frame.
    fn advance(&mut self);

    /// Reset to the first frame.
    fn reset(&mut self) {}

    /// Write ANSI codes to the given formatter
    /// that clears the printed output.
    #[allow(clippy::missing_errors_doc)]
    fn clear(&self, _f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }

    /// The amount of lines that is supposed to be printed
    /// and cleared.
    ///
    /// This is required by groups.
    fn lines(&self) -> usize {
        0
    }

    /// The length of the printed text if known
    /// in advance.
    fn print_len(&self) -> Option<usize> {
        None
    }
}
downcast::downcast_sync!(dyn Frames);

/// A stateful group of displayable frames
/// that are separated by new lines.
#[must_use]
#[derive(Default)]
pub struct Group {
    indent: usize,
    frames: Vec<Box<dyn Frames>>,
}

impl Group {
    /// Create a new empty group.
    pub fn new() -> Self {
        Self::default()
    }

    /// The amount of items in this group.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Whether the group has no children.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Insert an item at the given position.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    pub fn insert(&mut self, idx: usize, frames: impl Frames) -> &mut Self {
        self.frames.insert(idx, Box::new(frames));
        self
    }

    /// Push an item at the end.
    pub fn push(&mut self, frames: impl Frames) -> &mut Self {
        self.frames.push(Box::new(frames));
        self
    }

    /// Extend this group from an iterator.
    pub fn extend<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Frames,
    {
        for f in iter {
            self.push(f);
        }
        self
    }

    /// Remove an item at the given position.
    /// No-op if the position is invalid.
    pub fn remove(&mut self, idx: usize) -> &mut Self {
        self.frames.remove(idx);
        self
    }

    /// Return an iterator of the frames in this group.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Frames> + '_ {
        self.frames.iter().map(|s| &**s)
    }

    /// Return an iterator of the frames in this group.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Frames> + '_ {
        self.frames.iter_mut().map(|s| &mut **s)
    }

    /// Retains only the items specified by the predicate.
    pub fn retain(&mut self, f: impl Fn(&dyn Frames) -> bool) {
        self.frames.retain(move |s| f(&**s));
    }

    /// Print ANSI codes that clears the frames displayed
    /// by this group.
    ///
    /// # Example
    ///
    /// ```
    /// # use termspin::Group;
    /// # let group = Group::new();
    /// print!("{}", group.clear());
    /// ```
    #[must_use]
    pub fn clear(&self) -> impl core::fmt::Display + '_ {
        DisplayFn::new(|f| <Self as Frames>::clear(self, f))
    }

    /// Return the indentation level of this group.
    #[must_use]
    pub fn get_indent(&self) -> usize {
        self.indent
    }

    /// Set the indentation level of this group.
    pub fn with_indent(mut self, level: usize) -> Self {
        self.indent = level;
        self
    }

    /// Set the indentation level of this group.
    pub fn set_indent(&mut self, level: usize) -> &mut Self {
        self.indent = level;
        self
    }

    /// Turn this group into [`SharedFrames`].
    pub fn shared(self) -> SharedFrames<Self> {
        SharedFrames::new(self)
    }
}

impl core::fmt::Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for spinner in &self.frames {
            if spinner.lines() > 0 {
                for _ in 0..self.indent {
                    "  ".fmt(f)?;
                }
            }
            spinner.fmt(f)?;
            for _ in 0..spinner.lines() {
                '\n'.fmt(f)?;
            }
        }

        Ok(())
    }
}

impl Frames for Group {
    fn advance(&mut self) {
        for spinner in &mut self.frames {
            spinner.advance();
        }
    }

    fn reset(&mut self) {
        for spinner in &mut self.frames {
            spinner.reset();
        }
    }

    fn clear(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        for spinner in self.frames.iter().rev() {
            CursorUp(spinner.lines()).fmt(f)?;
            spinner.clear(f)?;
        }

        Ok(())
    }
}

/// A single line with a spinner and text.
#[must_use]
pub struct Line {
    show_spinner: bool,
    spinner: Box<dyn Frames>,
    text: Cow<'static, str>,
}

impl Line {
    /// Create a new line with the given spinner.
    pub fn new(spinner: impl Frames) -> Self {
        Self {
            spinner: Box::new(spinner),
            show_spinner: true,
            text: Cow::Borrowed(""),
        }
    }

    /// Get text that is displayed.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set the text that is displayed.
    pub fn set_text(&mut self, text: &str) -> &mut Self {
        self.text = text.to_string().into();
        self
    }

    /// Set the text that is displayed.
    pub fn with_text(mut self, text: &str) -> Self {
        self.text = text.to_string().into();
        self
    }

    /// Toggle the visibility of the spinner.
    pub fn set_spinner_visible(&mut self, show: bool) -> &mut Self {
        self.show_spinner = show;
        self
    }

    /// Toggle the visibility of the spinner.
    pub fn with_spinner_visible(mut self, show: bool) -> Self {
        self.show_spinner = show;
        self
    }

    /// Print ANSI codes that clears the frames displayed
    /// by this line.
    ///
    /// # Example
    ///
    /// ```
    /// # use termspin::Line;
    /// # let line = Line::new(termspin::spinner::from_iter([""]));
    /// print!("{}", line.clear());
    /// ```
    #[must_use]
    pub fn clear(&self) -> impl core::fmt::Display + '_ {
        DisplayFn::new(|f| <Self as Frames>::clear(self, f))
    }

    /// Turn this line into [`SharedFrames`].
    pub fn shared(self) -> SharedFrames<Self> {
        SharedFrames::new(self)
    }
}

impl Frames for Line {
    fn advance(&mut self) {
        self.spinner.advance();
    }
    fn reset(&mut self) {
        self.spinner.reset();
    }

    fn clear(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        "\r".fmt(f)?;
        ClearLine.fmt(f)
    }

    fn lines(&self) -> usize {
        1
    }
}

impl core::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.show_spinner {
            self.spinner.fmt(f)?;

            let spinner_printed = self.spinner.print_len().map_or(true, |l| l != 0);

            if spinner_printed && !self.text.is_empty() {
                f.write_char(' ')?;
            }
        }

        self.text.fmt(f)?;

        Ok(())
    }
}
