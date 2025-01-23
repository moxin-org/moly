#[cfg(not(any(feature = "async-rt", feature = "async-web")))]
compile_error!(
    "At least on of `async-rt` or `async-web` feature must be enabled to use `repos` feature"
);

pub mod moly;
