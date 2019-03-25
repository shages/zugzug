use std::error;
use std::fmt;

#[derive(Debug)]
pub struct ZugzugError {
    details: String,
}

impl ZugzugError {
    pub fn new(msg: &str) -> ZugzugError {
        ZugzugError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ZugzugError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl error::Error for ZugzugError {
    fn description(&self) -> &str {
        &self.details
    }
}
