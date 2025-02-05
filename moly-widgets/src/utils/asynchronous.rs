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
pub fn spawn(fut: impl Future<Output = ()> + 'static + Send) {
    tokio::task::spawn(fut);
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
