use gtk4::prelude::*;
use itertools::Itertools;

use crate::model::pg_response::{PgResponse, Table};

pub fn format_text(
    widget: &gtk4::TextView,
    batches: &[PgResponse],
    fmt_table_callback: fn(&Table) -> String,
) {
    let txt = batches
        .iter()
        .map(|batch| match batch {
            PgResponse::Table(table) => fmt_table_callback(table),
            PgResponse::CommandComplete(cc) => format!("rows_affected: {}", cc),
        })
        .join("\n");

    let buffer = widget.buffer();
    buffer.set_text(&txt);
}
