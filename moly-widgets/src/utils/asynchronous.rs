use std::future::Future;

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
