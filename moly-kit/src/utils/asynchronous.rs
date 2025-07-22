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

mod thread_token {
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_KEY: AtomicU64 = AtomicU64::new(0);

    thread_local! {
        static STORAGE: RefCell<HashMap<u64, Option<Box<dyn Any>>>> = RefCell::new(HashMap::new());
    }

    struct ThreadTokenInner<T: 'static> {
        key: u64,
        _phantom: std::marker::PhantomData<fn() -> T>,
    }

    impl<T> Drop for ThreadTokenInner<T> {
        fn drop(&mut self) {
            STORAGE.with_borrow_mut(|storage| {
                storage
                    .remove(&self.key)
                    .expect("Token dropped in a different thread.");
            });
        }
    }

    /// Holds a value inside a thread-local storage.
    ///
    /// Then, this token can be used to access the underlying value as long you
    /// are in the same thread that created it.
    ///
    /// This is useful on the web, where you are always in the same thread, but you
    /// need to pass some kind of non-`Send` value across `Send` boundries of Makepad.
    ///
    /// **Warning**: Trying to read the value from a different thread will panic.
    ///
    /// **Warning**: This token is reference counted so you can have copies of it,
    /// but the last copy must be dropped in the same thread that created it to
    /// avoid leaks. If this value is dropped in a different thread, it will panic.
    pub struct ThreadToken<T: 'static>(Arc<ThreadTokenInner<T>>);

    impl<T> Clone for ThreadToken<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }

    impl<T> std::fmt::Debug for ThreadToken<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "ThreadToken<{}>({})",
                std::any::type_name::<T>(),
                self.0.key
            )
        }
    }

    impl<T> ThreadToken<T> {
        /// Put the given value in thread-local storage and return a token to access it.
        pub fn new(value: T) -> Self {
            let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);

            STORAGE.with_borrow_mut(|storage| {
                storage.insert(key, Some(Box::new(value)));
            });

            Self(Arc::new(ThreadTokenInner {
                key,
                _phantom: std::marker::PhantomData,
            }))
        }

        /// Immutable access to the value associated with this token.
        pub fn peek<R>(&self, f: impl FnOnce(&T) -> R) -> R {
            STORAGE.with_borrow_mut(|storage| {
                let option = storage
                    .get(&self.0.key)
                    .expect("Token `peek` called from different thread");

                let boxed = option
                    .as_ref()
                    .expect("Token `peek` called after value was taken");

                let value = boxed.downcast_ref::<T>().unwrap();
                f(value)
            })
        }

        /// Mutable access to the value associated with this token.
        pub fn peek_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
            STORAGE.with_borrow_mut(|storage| {
                let option = storage
                    .get_mut(&self.0.key)
                    .expect("Token `peek_mut` called from different thread");

                let boxed = option
                    .as_mut()
                    .expect("Token `peek_mut` called after value was taken");

                let value = boxed.downcast_mut::<T>().unwrap();
                f(value)
            })
        }
    }

    impl<T: Clone> ThreadToken<T> {
        /// Clone the associated value of this token and return it.
        pub fn clone_inner(&self) -> T {
            self.peek(|value| value.clone())
        }
    }
}

pub use thread_token::*;
