pub mod telegram;

use crate::database::ReceiveMsg;

pub trait Integration: Send + Sync {
    fn integrate(&self, msg: ReceiveMsg) -> Box<dyn FnOnce() + Send + 'static>;
}

pub use telegram::Telegram;
