//! Asynchronous utilities for MolyKit.
//!
//! Mainly helps you to deal with the runtime differences across native and web.
//!
//! ## [spawn] function

//! Runs a future independently, in a platform-specific way.
//! - **Non-WASM**: May run in parallel and needs to be [Send].
//! - **WASM**: Will run concurrently and doesn't need to be [Send].
//! - **Android**: Creates a Tokio runtime if none exists (e.g., when running as a library).
//!
//! ## [MolyFuture] and [MolyStream]
//!
//! Dynamic and pinned wrappers around futures and streams with platform-specific implementations.
//! - **Non-WASM**: Requires [Send].
//! - **WASM**: Doesn't require [Send].
//!
//! ## [moly_future] and [moly_stream] functions
//!
//! Wraps a future or stream into a [MolyFuture] or [MolyStream] respectively.
//! - **Non-WASM**: Requires [Send].
//! - **WASM**: Doesn't require [Send].

// Note: I'm documenting functions and types in the module doc because I don't know
// right now an easy way to share the docs between conditional compilations.

// TODO: Continue thinking on a way to avoid this.
// The next thing to try would be to limit the native implementation to non-Send,
// and use `LocalSet` from Tokio as `reqwest` still needs Tokio on native.
//
// Note: `reqwest` gives you a `Send` future in native, but on web it uses a `JsValue`
// so its future is not send there.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    future::{Future, FutureExt},
    stream::{Stream, StreamExt},
};

#[cfg(feature = "async-rt")]
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;

#[cfg(feature = "async-rt")]
#[cfg(not(target_arch = "wasm32"))]
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

#[cfg(feature = "async-rt")]
#[cfg(not(target_arch = "wasm32"))]
fn get_or_create_runtime() -> &'static tokio::runtime::Runtime {
    RUNTIME.get_or_init(|| {
        log::info!("Creating Tokio runtime for MolyKit (likely running on Android or as library)");
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .thread_name("moly-tokio")
            .build()
            .expect("Failed to create Tokio runtime for MolyKit")
    })
}

/// Spawns a future to run independently.
///
/// This function handles different runtime contexts:
/// - If a Tokio runtime is already available, uses it directly
/// - If no runtime exists (e.g., Android/library context), creates a shared runtime
/// - On WASM, uses wasm-bindgen-futures
#[cfg(feature = "async-rt")]
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn(fut: impl Future<Output = ()> + 'static + Send) {
    // Try to spawn on existing runtime first, fallback to creating our own
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            log::trace!("Spawning on existing Tokio runtime");
            handle.spawn(fut);
        }
        Err(_) => {
            // No runtime available, use our own (common on Android)
            log::trace!("No Tokio runtime found, using MolyKit shared runtime");
            let runtime = get_or_create_runtime();
            runtime.spawn(fut);
        }
    }
}

#[cfg(feature = "async-web")]
#[cfg(target_arch = "wasm32")]
pub fn spawn(fut: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(fut);
}

#[cfg(not(target_arch = "wasm32"))]
pub struct MolyFuture<'a, T>(futures::future::BoxFuture<'a, T>);

#[cfg(not(target_arch = "wasm32"))]
pub struct MolyStream<'a, T>(futures::stream::BoxStream<'a, T>);

#[cfg(target_arch = "wasm32")]
pub struct MolyFuture<'a, T>(futures::future::LocalBoxFuture<'a, T>);

#[cfg(target_arch = "wasm32")]
pub struct MolyStream<'a, T>(futures::stream::LocalBoxStream<'a, T>);

#[cfg(not(target_arch = "wasm32"))]
pub fn moly_future<'a, T>(future: impl Future<Output = T> + 'a + Send) -> MolyFuture<'a, T> {
    MolyFuture(future.boxed())
}

#[cfg(target_arch = "wasm32")]
pub fn moly_future<'a, T>(future: impl Future<Output = T> + 'a) -> MolyFuture<'a, T> {
    MolyFuture(future.boxed_local())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn moly_stream<'a, T>(stream: impl Stream<Item = T> + 'a + Send) -> MolyStream<'a, T> {
    MolyStream(stream.boxed())
}

#[cfg(target_arch = "wasm32")]
pub fn moly_stream<'a, T>(stream: impl Stream<Item = T> + 'a) -> MolyStream<'a, T> {
    MolyStream(stream.boxed_local())
}

impl<'a, T> Future for MolyFuture<'a, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl<'a, T> Stream for MolyStream<'a, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}
