use crate::object_or_expect;
use gtk::prelude::*;

pub struct MainWindow {
    _window: gtk::ApplicationWindow,
}

impl MainWindow {
    pub fn new(window: gtk::ApplicationWindow) -> Self {
        Self { _window: window }
    }

    pub fn create(builder: &gtk::Builder, application: &gtk::Application) -> Self {
        let window: gtk::ApplicationWindow = object_or_expect(builder, "main_window");

        window.set_application(Some(application));

        window.set_default_size(1000, 800);
        window.show_all();

        MainWindow::new(window)
    }
}
