//! Extensions to [UiRunner] that make sense for moly kit.
//!
//! Async extensions could actually be part of [UiRunner] at makepad, but there is no
//! `futures` crate there, and idk if the async channels implemented there would work
//! propertly for this.

use makepad_widgets::{Cx, DeferWithRedraw, Scope, UiRunner, Widget};

pub trait DeferRedraw<W>
where
    Self: Sized,
{
    /// Requests to do a redraw and nothing else.
    ///
    /// Mostly a shorthand for `defer_with_redraw` with an empty closure.
    fn defer_redraw(self) {}
}

impl<W: Widget + 'static> DeferRedraw<W> for UiRunner<W> {
    fn defer_redraw(self) {
        self.defer_with_redraw(|_, _, _| {});
    }
}

pub trait AsyncDeferCallback<T, R>:
    FnOnce(&mut T, &mut Cx, &mut Scope) -> R + Send + 'static
where
    R: Send + 'static,
{
}

impl<T, R: Send + 'static, F: FnOnce(&mut T, &mut Cx, &mut Scope) -> R + Send + 'static>
    AsyncDeferCallback<T, R> for F
{
}

/// Async extension to [UiRunner], allowing to await until deferred closures are executed.
#[allow(unused)]
pub trait DeferAsync<T> {
    /// Awaitable variant of [UiRunner::defer].
    ///
    /// This is actually similar to [UiRunner::block_on], but can be used inside
    /// async contexts where is important to not block.
    ///
    /// The syncrhonous return value of the closure will be returned from the Future.
    ///
    /// The Future may give `None` if it couldn't receive a value back, because the
    /// widget coudln't execute the closure. This can happen even without errors if
    /// the widget is dropped before executing the pending closure.
    fn defer_async<R>(
        self,
        f: impl AsyncDeferCallback<T, R>,
    ) -> impl std::future::Future<Output = Option<R>> + Send
    where
        R: Send + 'static,
        Self: Sized;
}

impl<T: 'static> DeferAsync<T> for UiRunner<T> {
    async fn defer_async<R: Send + 'static>(self, f: impl AsyncDeferCallback<T, R>) -> Option<R> {
        let (tx, rx) = futures::channel::oneshot::channel::<R>();
        self.defer(move |me, cx, scope| {
            let _ = tx.send(f(me, cx, scope));
        });
        rx.await.ok()
    }
}

/// Async extension to [UiRunner], allowing to await until deferred closures with
/// redraw are executed
pub trait DeferWithRedrawAsync<T: 'static> {
    /// Awaitable variant of [DeferWithRedraw::defer_with_redraw] based on [DeferAsync::defer_async].
    ///
    /// Return value behaves the same as [DeferAsync::defer_async].
    fn defer_with_redraw_async<R>(
        self,
        f: impl AsyncDeferCallback<T, R>,
    ) -> impl std::future::Future<Output = Option<R>> + Send
    where
        R: Send + 'static,
        Self: Sized;
}

impl<W: Widget + 'static> DeferWithRedrawAsync<W> for UiRunner<W> {
    async fn defer_with_redraw_async<R: Send + 'static>(
        self,
        f: impl AsyncDeferCallback<W, R>,
    ) -> Option<R> {
        let (tx, rx) = futures::channel::oneshot::channel::<R>();
        self.defer_with_redraw(move |widget, cx, scope| {
            let _ = tx.send(f(widget, cx, scope));
        });
        rx.await.ok()
    }
}
