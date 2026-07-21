use std::error::Error;

pub type SnaprResult<T> = Result<T, Box<dyn Error + Send + Sync>>;