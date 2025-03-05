use std::sync::{Arc, Mutex};

use crate::capture::{CaptureHandler, Error};

pub fn register_handler<T>(_handler: Arc<Mutex<T>>) -> Result<(), Error>
where
    T: CaptureHandler,
{
    Ok(())
}
