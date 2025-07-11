//! Asynchronous utilities for MolyKit.
//!
//! Mainly helps you to deal with the runtime differences across native and web.
//!
//! For example: `reqwest` gives you a `Send` future in native, but on web it uses a `JsValue`
//! so its future is not send there.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    future::{AbortHandle, Abortable, Future, abortable},
    stream::Stream,
};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub trait PlatformSendInner {}
        impl<T> PlatformSendInner for T {}
    } else {
        pub trait PlatformSendInner: Send {}
        impl<T> PlatformSendInner for T where T: Send {}
    }
}

/// Implies [`Send`] only on native platforms, but not on WASM.
///
/// In other words:
/// - On native this gets implemented by all types that implement [`Send`].
/// - On WASM this gets implemented by all types, regardless of [`Send`].
pub trait PlatformSend: PlatformSendInner {}
impl<T> PlatformSend for T where T: PlatformSendInner {}

/// A future that requires [`Send`] on native platforms, but not on WASM.
pub trait PlatformSendFuture: Future + PlatformSend {}
impl<F, O> PlatformSendFuture for F where F: Future<Output = O> + PlatformSend {}

/// A stream that requires [`Send`] on native platforms, but not on WASM.
pub trait PlatformSendStream: Stream + PlatformSend {}
impl<S, T> PlatformSendStream for S where S: Stream<Item = T> + PlatformSend {}

/// Runs a future independently, in a platform-specific way.
///
/// - Uses tokio and requires [`Send`] on native platforms.
/// - Uses wasm-bindgen-futures on WASM and does not require [`Send`].
///
/// **Note:** This function may spawn it's own runtime if it can't find an existing one.
/// Currently, Makepad doesn't expose the entry point in certain platforms (like Android),
/// making harder to configure a runtime manually.
pub fn spawn(fut: impl PlatformSendFuture<Output = ()> + 'static) {
    spawn_impl(fut);
}

#[cfg(feature = "async-rt")]
#[cfg(not(target_arch = "wasm32"))]
fn spawn_impl(fut: impl Future<Output = ()> + 'static + Send) {
    use std::sync::OnceLock;
    use tokio::runtime::{Builder, Handle, Runtime};

    static RUNTIME: OnceLock<Runtime> = OnceLock::new();

    if let Ok(handle) = Handle::try_current() {
        handle.spawn(fut);
    } else {
        log::warn!("No Tokio runtime found on this native platform. Creating a shared runtime.");
        let rt = RUNTIME.get_or_init(|| {
            Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .thread_name("moly-kit-tokio")
                .build()
                .expect("Failed to create Tokio runtime for MolyKit")
        });
        rt.spawn(fut);
    }
}

#[cfg(feature = "async-web")]
#[cfg(target_arch = "wasm32")]
fn spawn_impl(fut: impl Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(fut);
}

/// A handle that aborts its associated future when dropped.
///
/// Similar to https://docs.rs/tokio-util/latest/tokio_util/task/struct.AbortOnDropHandle.html
/// but runtime agnostic.
///
/// This is created from the [`abort_on_drop`] function.
///
/// This is useful in Makepad to ensure tasks gets cancelled on widget drop instead
/// of keep running in the background unnoticed.
///
/// Note: In makepad, widgets may be cached or reused causing this to not work as expected
/// in many scenarios.
// TODO: Consider having a shared lightweight supervisor task that awakes makepad to check
// for responding handles through it's event system, but only if there are active tasks.
pub struct AbortOnDropHandle(AbortHandle);

impl Drop for AbortOnDropHandle {
    fn drop(&mut self) {
        self.abort();
    }
}

impl AbortOnDropHandle {
    /// Manually aborts the future associated with this handle before it is dropped.
    pub fn abort(&mut self) {
        self.0.abort();
    }
}

/// Constructs a future + [`AbortOnDropHandle`] pair.
///
/// See [`AbortOnDropHandle`] for more details.
pub fn abort_on_drop<F, T>(future: F) -> (Abortable<F>, AbortOnDropHandle)
where
    F: PlatformSendFuture<Output = T> + 'static,
{
    let (abort_handle, abort_registration) = abortable(future);
    (abort_handle, AbortOnDropHandle(abort_registration))
}

/// Opaque, boxed and pinned future commonly expected by traits in MolyKit.
///
/// This future requires [`Send`] only on native platforms, but not on WASM.
///
/// Use [`moly_future`] to create an instance of this type.
pub struct MolyFuture<'a, T>(Pin<Box<dyn PlatformSendFuture<Output = T> + 'a>>);
impl<'a, T> Future for MolyFuture<'a, T> {
    type Output = T;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

/// Opaque, boxed and pinned stream commonly expected by traits in MolyKit.
///
/// This stream requires [`Send`] only on native platforms, but not on WASM.
///
/// Use [`moly_stream`] to create an instance of this type.
pub struct MolyStream<'a, T>(Pin<Box<dyn PlatformSendStream<Item = T> + 'a>>);
impl<'a, T> Stream for MolyStream<'a, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.0).poll_next(cx)
    }
}

/// Wraps a future into a [`MolyFuture`].
pub fn moly_future<'a, T>(future: impl PlatformSendFuture<Output = T> + 'a) -> MolyFuture<'a, T> {
    MolyFuture(Box::pin(future))
}

/// Wraps a stream into a [`MolyStream`].
pub fn moly_stream<'a, T>(stream: impl PlatformSendStream<Item = T> + 'a) -> MolyStream<'a, T> {
    MolyStream(Box::pin(stream))
}
