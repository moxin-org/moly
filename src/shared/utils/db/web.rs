use super::Adapter;
use anyhow::Result;

#[derive(Clone, Default)]
pub(super) struct WebAdapter;

impl Adapter for WebAdapter {
    fn get(&mut self, key: &str) -> Result<Option<String>> {
        unimplemented!()
    }

    fn set(&mut self, key: &str, value: &str) -> Result<()> {
        unimplemented!()
    }

    fn has(&mut self, key: &str) -> Result<bool> {
        unimplemented!()
    }

    fn remove(&mut self, key: &str) -> Result<()> {
        unimplemented!()
    }

    fn keys(&mut self) -> Result<impl Iterator<Item = String>> {
        Ok(std::iter::empty::<String>())
    }
}
