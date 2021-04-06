use gtk::{prelude::BuilderExtManual, DialogExt, GtkWindowExt, WidgetExt};

use super::{AppEvent, Component, EventListener};

pub struct MainWindow {
    window: gtk::ApplicationWindow,
}

impl MainWindow {
    pub fn new(window: gtk::ApplicationWindow) -> Self {
        Self { window }
    }

    pub fn create(builder: &gtk::Builder, application: &gtk::Application) -> Self {
        let window: gtk::ApplicationWindow = builder
            .get_object("mainwin")
            .expect("Main window not present in window.ui");

        window.set_application(Some(application));

        MainWindow::new(window)
    }
}

impl Component for MainWindow {
    fn initialize(&self) {
        self.window.show_all();
    }

    fn children_mut(&mut self) -> Option<&mut [Box<dyn super::EventListener>]> {
        None
    }
}

impl EventListener for MainWindow {
    fn on_event(&self, event: &AppEvent) {
        if let AppEvent::Err(err) = event {
            let err_dialog = gtk::Dialog::with_buttons(
                Some(&err.to_string()),
                Some(&self.window),
                gtk::DialogFlags::MODAL,
                &[("Ok", gtk::ResponseType::Ok)],
            );

            err_dialog.connect_response(|d, _| d.close());

            err_dialog.show();
        }
    }
}
