use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod web;
        pub use web::*;
    } else {
        mod native;
        pub use native::*;
    }
}
