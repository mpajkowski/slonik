use crate::{
    event::{AppEvent, EventListener},
    object_or_expect,
};
use glib::clone;
use gtk::prelude::*;

pub struct MainWindow {
    window: gtk::ApplicationWindow,
    pane_horizontal: gtk::Paned,
    pane_vertical: gtk::Paned,
}

impl MainWindow {
    pub fn new(
        window: gtk::ApplicationWindow,
        pane_horizontal: gtk::Paned,
        pane_vertical: gtk::Paned,
    ) -> Self {
        Self {
            window,
            pane_horizontal,
            pane_vertical,
        }
    }

    pub fn create(builder: &gtk::Builder, application: &gtk::Application) -> Self {
        let window: gtk::ApplicationWindow = object_or_expect(builder, "main_window");
        let pane_horizontal: gtk::Paned = object_or_expect(builder, "pane_horizontal");
        let pane_vertical: gtk::Paned = object_or_expect(builder, "pane_vertical");

        window.set_application(Some(application));

        window.connect_show(|w| println!("{:?}", w.get_size()));
        window.set_default_size(1000, 800);
        window.show_all();

        pane_horizontal.connect_check_resize(clone!(@strong window => move |pane| {
            let (x, _) = window.get_size();
            println!("X: {}", x);
        }));

        MainWindow::new(window, pane_horizontal, pane_vertical)
    }
}
