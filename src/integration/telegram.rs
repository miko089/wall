use crate::database::ReceiveMsg;
use crate::integration::Integration;
use crate::utils::html::escape_html;
use serde_json::json;

pub struct Telegram {
    token: String,
    chat_id: String,
}

impl Telegram {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }
}

impl Integration for Telegram {
    fn integrate(&self, msg: ReceiveMsg) -> Box<dyn FnOnce() + Send + 'static> {
        let author = escape_html(msg.author.as_ref());
        let content = escape_html(msg.content.as_ref());

        let url = format!(
            "https://api.telegram.org/bot{}/sendMessage",
            self.token
        );
        let chat_id = self.chat_id.clone();
        let text = format!("<b>{}</b>:\n{}", author, content);
        tracing::info!("Sending a message to Telegram (chat: {}): {}", self.chat_id, text);
        Box::new(move || match ureq::post(&url)
            .content_type("application/json")
            .send_json(&json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "HTML",
            })) {
            Ok(_) => {}
            Err(e) => tracing::error!("Failed to send a message to Telegram: {}", e),
        })
    }
}
