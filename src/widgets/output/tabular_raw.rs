use super::{output_mode::OutputMode, textutils::format_text};
use crate::model::pg_response::{PgResponse, Table};

#[derive(Debug)]
pub struct TabularRawOutputMode {
    widget: gtk4::TextView,
}

impl OutputMode for TabularRawOutputMode {
    fn format_batches(&self, batches: &[PgResponse]) {
        format_text(&self.widget, batches, format_raw);
    }

    fn create(parent: &gtk4::ScrolledWindow) -> Self
    where
        Self: Sized,
    {
        let widget = gtk4::TextViewBuilder::new()
            .editable(false)
            .monospace(true)
            .build();
        parent.set_child(Some(&widget));

        Self { widget }
    }
}

fn format_raw(table: &Table) -> String {
    let mut prettytable = prettytable::Table::new();
    prettytable.add_row(table.header.columns.clone().into());
    table
        .rows
        .iter()
        .map(|row| {
            row.values
                .iter()
                .map(|x| x.as_deref().unwrap_or("[null]"))
                .collect::<Vec<_>>()
        })
        .for_each(|row| {
            prettytable.add_row(row.into());
        });

    prettytable.to_string()
}
