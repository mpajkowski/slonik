use crate::event::{AppAction, AppEvent, Emitter, EventListener, PgRequest};
use gtk4::prelude::*;

pub struct Editor {
    _widget: sourceview5::View,
    buffer: sourceview5::Buffer,
    emitter: Emitter,
}

impl Editor {
    pub fn create(parent: &gtk4::ScrolledWindow, emitter: Emitter) -> Self {
        let lang_mgr = sourceview5::LanguageManager::new();
        let sql = lang_mgr.language("sql");
        let style_scheme_mgr = sourceview5::StyleSchemeManager::new();
        let style_scheme = style_scheme_mgr.scheme("oblivion").unwrap();

        let buffer = sourceview5::Buffer::builder()
            .highlight_syntax(true)
            .highlight_matching_brackets(true)
            .language(&sql.unwrap())
            .enable_undo(true)
            .style_scheme(&style_scheme)
            .build();

        let widget = sourceview5::ViewBuilder::new()
            .editable(true)
            .monospace(true)
            .show_line_numbers(true)
            .highlight_current_line(true)
            .visible(true)
            .insert_spaces_instead_of_tabs(true)
            .tab_width(2)
            .indent(2)
            .auto_indent(true)
            .wrap_mode(gtk4::WrapMode::None)
            .buffer(&buffer)
            .build();

        parent.set_child(Some(&widget));

        Self {
            _widget: widget,
            buffer,
            emitter,
        }
    }
}

impl EventListener for Editor {
    fn on_event(&mut self, event: &AppEvent) {
        if let AppEvent::AppAction(AppAction::FetchRows) = event {
            let buffer = &self.buffer;

            let (begin, end) = if buffer.has_selection() {
                buffer.selection_bounds().unwrap()
            } else {
                buffer.bounds()
            };

            let text = buffer.text(&begin, &end, false);
            self.emitter.emit(AppEvent::PgRequest(PgRequest {
                id: 0,
                text: text.into(),
            }));
        }
    }
}
