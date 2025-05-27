use anyhow::Result;
use std::future::Future;
use std::path::Path;

pub trait Adapter: Send + Sync + 'static {
    fn write(&mut self, path: &Path, content: &[u8]) -> impl Future<Output = Result<()>> + Send;
    fn read(&mut self, path: &Path) -> impl Future<Output = Result<Vec<u8>>> + Send;
    fn exists(&mut self, path: &Path) -> impl Future<Output = Result<bool>> + Send;
    fn remove(&mut self, path: &Path) -> impl Future<Output = Result<()>> + Send;
    fn list(&mut self, path: &Path) -> impl Future<Output = Result<Vec<String>>> + Send;
}
