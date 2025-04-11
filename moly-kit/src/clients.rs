// TODO: Maybe `json` feature flag can be avoided by using Makepad's microserde.
#[cfg(feature = "json")]
pub mod openai;
pub mod multi;
pub mod deep_inquire;

#[cfg(feature = "json")]
pub use openai::*;
pub use multi::*;
pub use deep_inquire::*;
