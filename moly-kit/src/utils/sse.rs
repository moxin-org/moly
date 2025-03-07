//! Utilities to deal with SSE (Server-Sent Events).
//!
//! Note: Eventually, a proper SSE parser should be here but for now it only contains
//! utility functions to do the parsing somewhere else.

pub(crate) const EVENT_TERMINATOR: &'static [u8] = b"\n\n";

/// Split from the last SSE event terminator.
///
/// On the left side you will get the side of the buffer that contains completed messages.
/// Although, the last terminator has been removed, this side may still contain multiple
/// messages that need to be split.
///
/// On the right side you will get the side of the buffer that contains the incomplete data,
/// so you should keep this on the buffer until completed.
///
/// Returns `None` if no terminator is found.
pub(crate) fn rsplit_once_terminator(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
    buffer
        .windows(2)
        .enumerate()
        .rev()
        .find(|(_, w)| w == &EVENT_TERMINATOR)
        .map(|(pos, _)| {
            let (before, after_with_terminator) = buffer.split_at(pos);
            let after = &after_with_terminator[2..];
            (before, after)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsplit_once_terminator() {
        let buffer = b"data: 1\n\ndata: 2\n\ndata: incomplete mes";
        let (completed, incomplete) = rsplit_once_terminator(buffer).unwrap();
        assert_eq!(completed, b"data: 1\n\ndata: 2");
        assert_eq!(incomplete, b"data: incomplete mes");
    }
}
