pub mod debug_logger;
pub mod event;
pub mod model;
pub mod pg_session;
pub mod widgets;

use anyhow::{bail, Result};
use gio::prelude::*;
use glib::{Object, PRIORITY_HIGH_IDLE};
use gtk::{prelude::*, Builder};
use mimalloc::MiMalloc;
use tokio::runtime::Runtime;

use crate::{
    debug_logger::DebugLogger, event::DispatchLoop, pg_session::PgEventLoopProxy, widgets::Editor,
};
use widgets::MainWindow;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    gtk::init()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let application =
        gtk::Application::new(Some("com.github.mpajkowski.slonik"), Default::default())?;

    application.connect_activate(move |app| {
        let runtime = &runtime;
        build_app(runtime, app);
    });

    let ret = application.run(&std::env::args().collect::<Vec<_>>());

    if ret == 0 {
        Ok(())
    } else {
        bail!("gtk retcode: {}", ret)
    }
}

fn build_app(runtime: &Runtime, app: &gtk::Application) {
    let glade_src = include_str!("../resources/window.ui");
    let builder = Builder::from_string(glade_src);
    let _guard = runtime.enter();
    let ctx = glib::MainContext::default();
    ctx.push_thread_default();

    let mut dispatch_loop = DispatchLoop::create();
    let _ = runtime.enter();
    dispatch_loop.register_listener(DebugLogger);
    dispatch_loop.register_listener(PgEventLoopProxy::initialize(dispatch_loop.create_emitter()));
    dispatch_loop.register_listener(widgets::Output::create(
        &builder,
        dispatch_loop.create_emitter(),
    ));
    dispatch_loop.register_listener(widgets::Messages::create(&builder));

    let _editor = Editor::create(&builder, dispatch_loop.create_emitter());
    let _main_window = MainWindow::create(&builder, app);

    ctx.spawn_local_with_priority(PRIORITY_HIGH_IDLE, dispatch_loop.listen());
}

pub fn object_or_expect<T: IsA<Object>>(builder: &gtk::Builder, object_name: &str) -> T {
    builder
        .get_object(object_name)
        .unwrap_or_else(|| panic!("{} not found", object_name))
}
