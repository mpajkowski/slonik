use gdk::EventType;
use gtk::prelude::*;
use sourceview::prelude::*;
use sourceview::LanguageManagerExt;

use crate::{
    event::{AppEvent, Emitter, PgRequest},
    object_or_expect,
};

pub struct Editor {
    _widget: sourceview::View,
    _emitter: Emitter,
}

impl Editor {
    pub fn create(builder: &gtk::Builder, emitter: Emitter) -> Self {
        let widget: sourceview::View = object_or_expect(builder, "editor");

        // FIXME plpgsql does not work, sql is used as a  fallback
        let lang_mgr = sourceview::LanguageManager::new();
        let mut search_path = lang_mgr
            .get_search_path()
            .into_iter()
            .map(|gstring| gstring.to_string())
            .collect::<Vec<_>>();
        search_path.push("/home/marcin/proj/slonik/resources/pgsql.lang".into());
        lang_mgr.set_search_path(&search_path.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let sql = lang_mgr
            .get_language("pgsql")
            .or_else(|| lang_mgr.get_language("sql"))
            .unwrap();

        let buffer = widget.get_buffer().unwrap();
        let buffer = buffer.downcast::<sourceview::Buffer>().unwrap();
        buffer.set_language(Some(&sql));

        {
            let emitter = emitter.clone();
            widget.connect_key_press_event(move |widget, key_event| {
                if key_event.get_event_type() == EventType::KeyPress
                    && key_event.get_keyval() == gdk::keys::constants::F5
                    && widget.is_focus()
                {
                    let buffer = widget.get_buffer().unwrap();

                    let (begin, end) = if buffer.get_has_selection() {
                        buffer.get_selection_bounds().unwrap()
                    } else {
                        buffer.get_bounds()
                    };

                    let text = buffer.get_text(&begin, &end, false).unwrap();
                    emitter.emit(AppEvent::PgRequest(PgRequest {
                        id: 0,
                        text: text.into(),
                    }));
                }

                Inhibit(false)
            });
        }

        Self {
            _widget: widget,
            _emitter: emitter,
        }
    }
}
