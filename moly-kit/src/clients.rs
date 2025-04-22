cfg_if::cfg_if! {
    // TODO: Maybe `json` feature flag can be avoided by using Makepad's microserde.
    if #[cfg(all(feature = "json", feature = "http"))] {
        pub mod openai;
        pub use openai::OpenAIClient;

        pub mod deep_inquire;
        pub use deep_inquire::DeepInquireClient;
    }
}

pub use multi::*;
pub mod multi;
