use crate::capture::{CaptureHandler, InitError};

pub fn register_handler<T>(handler: T) -> Result<(), InitError>
where
    T: CaptureHandler,
{
    Ok(())
}
