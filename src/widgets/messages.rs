use std::ops::Deref;

use gtk4::prelude::*;
use tokio_postgres::AsyncMessage;

use crate::{
    event::{AppEvent, EventListener},
    object_or_expect,
};

pub struct Messages {
    widget: gtk4::TextView,
}

impl Messages {
    pub fn create(builder: &gtk4::Builder) -> Self {
        let widget: gtk4::TextView = object_or_expect(builder, "messages");
        widget.set_monospace(true);

        Self { widget }
    }
}

impl EventListener for Messages {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::PgMessage(msg) = event {
            let buf = self.widget.buffer();
            let new_buffer = match msg.deref() {
                AsyncMessage::Notice(notice) => notice.to_string(),
                AsyncMessage::Notification(notifcation) => {
                    let channel = notifcation.channel();
                    let payload = notifcation.payload();
                    let pid = notifcation.process_id();

                    format!("[pid={}, channel={}] {}", pid, channel, payload)
                }
                _ => unreachable!(),
            };

            buf.set_text(&new_buffer);
        }
    }
}
