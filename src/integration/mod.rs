#[cfg(feature = "integration_tg")]
pub mod telegram;

use crate::database::ReceiveMsg;

pub trait Integration: Send + Sync {
    fn integrate(&self, msg: ReceiveMsg) -> Box<dyn FnOnce() + Send + 'static>;
    fn parse_args(&mut self) -> anyhow::Result<()>;
}

#[cfg(feature = "integration_tg")]
pub use telegram::Telegram;
