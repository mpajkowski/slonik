pub mod model;

use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

pub use self::model::NavigatorModel;

use super::{Component, EventListener};

pub struct Navigator {
    uri_txt: gtk::Entry,
    get_button: gtk::Button,
    model: Rc<NavigatorModel>,
}

impl Navigator {
    pub fn new(uri_txt: gtk::Entry, get_button: gtk::Button, model: Rc<NavigatorModel>) -> Self {
        Self {
            uri_txt,
            get_button,
            model,
        }
    }

    pub fn create(builder: &gtk::Builder, model: Rc<NavigatorModel>) -> Self {
        let uri_txt: gtk::Entry = builder
            .object("url_input")
            .expect("URI input not present in window.gradle");

        let get_button: gtk::Button = builder
            .object("get_button")
            .expect("URI input not present in window.gradle");

        Navigator::new(uri_txt, get_button, model)
    }
}

impl Component for Navigator {
    fn children_mut(&mut self) -> Option<&mut [Box<dyn EventListener>]> {
        None
    }

    fn initialize(&self) {
        use gtk::gdk::keys::constants as kk;

        let model = Rc::clone(&self.model);
        self.get_button
            .connect_clicked(glib::clone!(@weak self.uri_txt as uri_txt => move |_| {
                model.retrieve_headers(&uri_txt.text()).unwrap()
            }));

        let model = Rc::clone(&self.model);
        self.uri_txt.connect_key_press_event(
            glib::clone!(@weak self.uri_txt as uri_txt => @default-return Inhibit(false), move |_, k| {
                let key = k.keyval();

                log::debug!("nav key: {}", key);

                if key == kk::Return {
                    model.retrieve_headers(&uri_txt.text()).unwrap();
                }

                Inhibit(false)
            }),
        );
    }
}
