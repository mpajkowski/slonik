use gtk::{TextBufferExt, TextViewExt};

use crate::{
    event::{AppEvent, EventListener},
    object_or_expect,
};

pub struct Messages {
    widget: gtk::TextView,
}

impl Messages {
    pub fn create(builder: &gtk::Builder) -> Self {
        let widget: gtk::TextView = object_or_expect(builder, "messages");
        widget.set_monospace(true);

        Self { widget }
    }
}

impl EventListener for Messages {
    fn on_event(&self, event: &AppEvent) {
        todo!()
    }
}
