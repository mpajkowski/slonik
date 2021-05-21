pub mod components;
mod emit;
pub mod worker;

use std::rc::Rc;

use components::{Component, MainWindow, Navigator, NavigatorModel, TextView};

use anyhow::Result;
use emit::DispatchLoop;
use gtk::{
    glib::Priority,
    prelude::{ApplicationExt, ApplicationExtManual},
    Builder,
};
use tokio::runtime::Runtime;
use worker::Worker;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    gtk::init()?;

    let application =
        gtk::Application::new(Some("com.github.mpajkowski.slonik"), Default::default());

    application.connect_activate(move |app| build_app(app));
    application.run();

    Ok(())
}

fn build_app(app: &gtk::Application) {
    let glade_src = include_str!("../resources/window.ui");
    let builder = Builder::from_string(glade_src);

    let ctx = gtk::glib::MainContext::default();

    let mut dispatch_loop = DispatchLoop::create();
    let worker =
        Worker::create(dispatch_loop.create_emitter()).expect("Failed to initialize worker");

    let main_window = MainWindow::create(&builder, app);
    let nav_model = NavigatorModel::new(worker, dispatch_loop.create_emitter());
    let navigator = Navigator::create(&builder, Rc::new(nav_model));
    let text_view = TextView::create(&builder);

    main_window.initialize();
    navigator.initialize();
    text_view.initialize();

    dispatch_loop.register_listener(Box::new(text_view));
    dispatch_loop.register_listener(Box::new(main_window));

    ctx.spawn_local_with_priority(
        gtk::glib::source::PRIORITY_DEFAULT_IDLE,
        dispatch_loop.listen(),
    );
}
