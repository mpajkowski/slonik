use super::{AppEvent, Component, EventListener};
use gtk::prelude::{BuilderExtManual, TextBufferExt, TextViewExt};

pub struct TextView {
    text_view: gtk::TextView,
}

impl TextView {
    pub fn create(builder: &gtk::Builder) -> Self {
        let text_view: gtk::TextView = builder
            .object("text_view")
            .expect("text_view not present in window.ui");

        Self::new(text_view)
    }

    pub fn new(text_view: gtk::TextView) -> Self {
        Self { text_view }
    }
}

impl Component for TextView {
    fn children_mut(&mut self) -> Option<&mut [Box<dyn super::EventListener>]> {
        None
    }

    fn initialize(&self) {
        self.text_view.set_editable(false);
    }
}

impl EventListener for TextView {
    fn on_event(&self, event: &AppEvent) {
        if let AppEvent::Contents { contents } = event {
            let buf = self.text_view.buffer().unwrap();
            buf.set_text(contents);
        }
    }
}
