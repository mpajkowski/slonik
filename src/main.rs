pub mod debug_logger;
pub mod event;
pub mod model;
pub mod pg_session;
pub mod widgets;

use anyhow::{bail, Result};
use event::{AppAction, AppEvent, Emitter};
use gdk4::gio::SimpleAction;
use glib::{clone, Object, PRIORITY_HIGH_IDLE};
use gtk4::{prelude::*, Builder};
use mimalloc::MiMalloc;
use tokio::runtime::Runtime;

use crate::{
    debug_logger::DebugLogger, event::EventDispatcher, pg_session::PgEventLoopProxy,
    widgets::Editor,
};
use widgets::MainWindow;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    gtk4::init()?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let application =
        gtk4::Application::new(Some("com.github.mpajkowski.slonik"), Default::default());

    application.connect_activate(move |app| {
        let runtime = &runtime;
        build_app(runtime, app);
    });

    let ret = application.run();

    if ret == 0 {
        Ok(())
    } else {
        bail!("gtk retcode: {}", ret)
    }
}

fn build_app(runtime: &Runtime, app: &gtk4::Application) {
    let glade_src = include_str!("../resources/window.gtk4.ui");
    let builder = Builder::from_string(glade_src);
    let _guard = runtime.enter();
    let ctx = glib::MainContext::default();
    ctx.push_thread_default();

    let mut event_dispatcher = EventDispatcher::create();
    let _ = runtime.enter();

    register_actions(app, event_dispatcher.create_emitter());

    event_dispatcher.register_listener(DebugLogger);
    event_dispatcher.register_listener(PgEventLoopProxy::initialize(
        event_dispatcher.create_emitter(),
    ));
    event_dispatcher.register_listener(widgets::Output::create(
        &builder,
        event_dispatcher.create_emitter(),
    ));
    event_dispatcher.register_listener(widgets::Messages::create(&builder));

    let _main_window = MainWindow::create(&builder, app);
    let editor_parent = object_or_expect(&builder, "scrolled_editor");
    let editor = Editor::create(&editor_parent, event_dispatcher.create_emitter());
    event_dispatcher.register_listener(editor);

    ctx.spawn_local_with_priority(PRIORITY_HIGH_IDLE, event_dispatcher.listen());
}

pub fn object_or_expect<T: IsA<Object>>(builder: &gtk4::Builder, object_name: &str) -> T {
    builder
        .object(object_name)
        .unwrap_or_else(|| panic!("'{}' not found", object_name))
}

pub fn register_actions(app: &gtk4::Application, emitter: Emitter) {
    let quit = SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak app => move |_,_| app.quit()));
    app.add_action(&quit);
    app.set_accels_for_action("app.quit", &["<Ctrl>q"]);

    let fetch_rows = make_action("fetch_rows", AppAction::FetchRows, emitter);
    app.add_action(&fetch_rows);
    app.set_accels_for_action("app.fetch_rows", &["F5"]);
}

fn make_action(name: &str, app_action: AppAction, emitter: Emitter) -> SimpleAction {
    let action = SimpleAction::new(name, None);
    action.connect_activate(move |_, _| emitter.emit(AppEvent::AppAction(app_action)));
    action
}
