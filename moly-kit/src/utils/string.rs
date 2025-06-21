/// Yields more and more of the string on each iteration, with a recommended chunk size.
///
/// String is obtained as a whole.
#[derive(Debug)]
pub(crate) struct AmortizedString {
    text: String,
    tail: usize,

    /// Only set after the whole string has been consumed, but reseted on update.
    is_done: bool,
}

impl Default for AmortizedString {
    fn default() -> Self {
        Self {
            text: String::new(),
            tail: 0,
            // Guarantee that the first iteration will yield something.
            is_done: true,
        }
    }
}

impl Iterator for AmortizedString {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        self.tail = (self.tail + Self::CHUNK_SIZE).min(self.text.len());

        // Let's correct the tail if not at a char boundary or start/end of
        // the buffer.
        while !self.text.is_char_boundary(self.tail) {
            self.tail += 1;
        }

        self.is_done = self.tail == self.text.len();
        Some(self.current().to_string())
    }
}

impl AmortizedString {
    // Factors
    /// How many characters to send at once.
    ///
    /// - `1` gives the smoothest experience, but will slow down the consumer end
    ///   of the stream and also cause more redraws on makepad's side.
    /// - `10`, on a model that yields big chunks of text like Gemini-1.5-flash,
    ///   will look decent, but not perfectly smooth.
    /// - `5` is a good compromise between smoothness and performance.
    ///
    /// TODO: Use unicode segmentation instead of byte indexes, so this can be
    /// 5 real characters instead of 2-3 in multi-byte languages like Chinese.
    const CHUNK_SIZE: usize = 5;

    /// Peek at the current slice of the string until the current iteration's tail.
    pub(crate) fn current(&self) -> &str {
        &self.text[..self.tail]
    }

    /// Feed a new string to keep iterating over it.
    ///
    /// If the new string it's just the previous string with more text appended,
    /// everything will continue as before.
    ///
    /// However, if the new string doesn't start with the previous string, the next
    /// iteration will return the whole modified string.
    ///
    /// Upon calling this, the iterator is guaranteed to at least yield one more time.
    pub(crate) fn update(&mut self, text: String) {
        if !text.starts_with(&self.text) {
            self.tail = text.len();
        }

        self.text = text;
        self.is_done = false;
    }
}
