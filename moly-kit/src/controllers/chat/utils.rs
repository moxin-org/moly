use crate::{protocol::*, utils::asynchronous::*};

/// Util that wraps the stream of `send()` and gives you a stream less agresive to
/// the receiver UI regardless of the streaming chunk size.
pub(super) fn amortize(
    input: impl PlatformSendStream<Item = ClientResult<MessageContent>> + 'static,
) -> impl PlatformSendStream<Item = ClientResult<MessageContent>> + 'static {
    // Use utils
    use crate::utils::string::AmortizedString;
    use async_stream::stream;

    // Stream state
    let mut amortized_text = AmortizedString::default();
    let mut amortized_reasoning = AmortizedString::default();

    // Stream compute
    stream! {
        // Our wrapper stream "activates" when something comes from the underlying stream.
        for await result in input {
            // Transparently yield the result on error and then stop.
            if result.has_errors() {
                yield result;
                return;
            }

            // Modified content that we will be yielding.
            let mut content = result.into_value().unwrap();

            // Feed the whole string into the string amortizer.
            // Put back what has been already amortized from previous iterations.
            let text = std::mem::take(&mut content.text);
            amortized_text.update(text);
            content.text = amortized_text.current().to_string();

            // Same for reasoning.
            let reasoning = std::mem::take(&mut content.reasoning);
            amortized_reasoning.update(reasoning);
            content.reasoning = amortized_reasoning.current().to_string();

            // Prioritize yielding amortized reasoning updates first.
            for reasoning in &mut amortized_reasoning {
                content.reasoning = reasoning;
                yield ClientResult::new_ok(content.clone());
            }

            // Finially, begin yielding amortized text updates.
            // This will also include the amortized reasoning until now because we
            // fed it back into the content.
            for text in &mut amortized_text {
                content.text = text;
                yield ClientResult::new_ok(content.clone());
            }
        }
    }
}
