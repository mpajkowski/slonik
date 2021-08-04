use crate::model::pg_response::{PgResponse, Table};

use super::output_mode::OutputMode;
use glib::ToValue;
use gtk4::{glib::Type, prelude::*};
use gtk4::{CellRendererText, Label, ListStore, TextViewBuilder, TreeView, TreeViewColumn};

#[derive(Debug)]
pub struct TabularOutputMode {
    widget: gtk4::ListBox,
}

impl OutputMode for TabularOutputMode {
    fn create(parent: &gtk4::ScrolledWindow) -> Self
    where
        Self: Sized,
    {
        let listbox = gtk4::ListBox::new();

        parent.set_child(Some(&listbox));

        Self { widget: listbox }
    }

    fn format_batches(&self, batches: &[PgResponse]) {
        // remove all previously formatted children
        while let Some(child) = self.widget.last_child() {
            self.widget.remove(&child);
        }

        for batch in batches {
            let child = match batch {
                PgResponse::Table(t) => format_table(t),
                PgResponse::CommandComplete(cc) => format_cc(*cc),
            };

            self.widget.append(&child);
        }

        self.widget.show();
    }
}

fn format_table(table: &Table) -> gtk4::Widget {
    let columns_len = table.header.columns.len();
    let model_ty = (0..columns_len).map(|_| Type::STRING).collect::<Vec<_>>();

    let model_ty = model_ty.as_slice();

    let store = ListStore::new(model_ty);

    for row in table.rows.iter() {
        let iter = store.append();
        row.values.iter().enumerate().for_each(|(idx, v)| {
            store.set_value(
                &iter,
                idx as u32,
                &v.as_deref().unwrap_or("[null]").to_value(),
            )
        });
    }

    let view = TreeView::new();
    view.set_headers_visible(true);
    view.set_model(Some(&store));

    for (idx, name) in table.header.columns.iter().enumerate() {
        let column = TreeViewColumn::new();
        let cell = CellRendererText::new();

        column.pack_start(&cell, true);
        // Association of the view's column with the model's `id` column.
        column.add_attribute(&cell, "text", idx as _);
        column.set_widget(Some(&Label::new(Some(name))));
        view.append_column(&column);
    }
    view.show();

    view.upcast()
}

fn format_cc(rows_affected: u64) -> gtk4::Widget {
    let widget = TextViewBuilder::new()
        .editable(false)
        .monospace(true)
        .build();

    let buf = widget.buffer();
    buf.set_text(&format!("Rows affected: {}", rows_affected));

    widget.upcast()
}
