use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_LOOPS: usize = 10;
pub const DEBUG_LOG: bool = false;
pub const WAITING_MS: u64 = 10;

#[derive(Serialize, Deserialize, Debug)]
pub struct SimplePayload {
    pub x: u128,
    pub y: u128,
    pub millis: u128,
}

impl SimplePayload {
    pub fn new() -> Self {
        let curr_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Computer clock is after 1970");
        let millis = curr_time.as_millis();
        Self {
            x: millis / 60,
            y: millis / (60 * 60),
            millis,
        }
    }
}

impl ToString for SimplePayload {
    fn to_string(&self) -> String {
        serde_json::to_string(self).expect("Failed serializing to string")
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if extended::common::DEBUG_LOG {
            println!($($arg)*);
        }
    };
}

pub use log;
