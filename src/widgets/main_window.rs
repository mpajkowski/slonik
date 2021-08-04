use crate::object_or_expect;
use gtk4::prelude::*;

pub struct MainWindow {
    _window: gtk4::ApplicationWindow,
}

impl MainWindow {
    pub fn new(window: gtk4::ApplicationWindow) -> Self {
        Self { _window: window }
    }

    pub fn create(builder: &gtk4::Builder, application: &gtk4::Application) -> Self {
        let window: gtk4::ApplicationWindow = object_or_expect(builder, "main_window");

        window.set_application(Some(application));

        window.set_default_size(1000, 800);
        window.show();

        MainWindow::new(window)
    }
}
