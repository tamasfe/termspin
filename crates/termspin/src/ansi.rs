//! Helpers for ANSI escape codes.

/// Move the cursor up a line.
pub struct CursorUp(pub usize);

impl core::fmt::Display for CursorUp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 > 0 {
            write!(f, "\x1B[{}A", self.0)?;
        }
        Ok(())
    }
}

/// Clear the current line.
pub struct ClearLine;

impl core::fmt::Display for ClearLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("\x1B[2K")
    }
}
