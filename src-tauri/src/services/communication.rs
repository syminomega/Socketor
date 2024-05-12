use tauri::Window;

#[derive(Clone, serde::Serialize)]
pub struct MessageItem {
    pub content: String,
    pub msg_type: MessageType,
}

#[derive(Clone, serde::Serialize)]
pub enum MessageType {
    Log,
    Send,
    Receive,
}

pub fn show_message(message: &str, msg_type: MessageType, window: &Window) {
    let message = MessageItem {
        content: message.to_string(),
        msg_type,
    };
    match window.emit("show-message",message) {
        Ok(_) => {
            // println!("send message to front");
        }
        Err(e) => {
            println!("send message to front error:{}", e);
        }
    }
}

#[derive(Clone,serde::Serialize)]
pub struct ActionResult{
    pub success: bool,
    pub error_message: String,
}