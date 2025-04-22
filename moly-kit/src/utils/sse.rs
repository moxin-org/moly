//! Utilities to deal with SSE (Server-Sent Events).
//!
//! Note: Eventually, a proper SSE parser should be here but for now it only contains
//! utility functions to do the parsing somewhere else.

use async_stream::stream;
use futures::Stream;

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

/// Convert a stream of bytes into a stream of SSE messages.
pub(crate) fn parse_sse<S, B, E>(s: S) -> impl Stream<Item = Result<String, E>>
where
    S: Stream<Item = Result<B, E>>,
    B: AsRef<[u8]>,
{
    stream! {
        let event_terminator_str = std::str::from_utf8(EVENT_TERMINATOR).unwrap();
        let mut buffer: Vec<u8> = Vec::new();

        for await chunk in s {
            let chunk = match chunk {
                Ok(chunk) => chunk,
                Err(error) => {
                    yield Err(error);
                    return;
                }
            };

            let chunk = chunk.as_ref();

            buffer.extend_from_slice(chunk);

            let Some((completed_messages, incomplete_message)) =
                rsplit_once_terminator(&buffer)
            else {
                continue;
            };

            // Silently drop any invalid utf8 bytes from the completed messages.
            let completed_messages = String::from_utf8_lossy(completed_messages);

            let messages =
                completed_messages
                .split(event_terminator_str)
                .filter(|m| !m.starts_with(":"))
                // TODO: Return a format error instead of unwraping.
                .map(|m| m.trim_start().split("data:").nth(1).unwrap())
                .filter(|m| m.trim() != "[DONE]");

            for m in messages {
                yield Ok(m.to_string());
            }

            buffer = incomplete_message.to_vec();
        }
    }
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
