use crate::{event::OutputModeChange, model::pg_response::PgResponse};

use super::{csv::CsvOutputMode, tabular_raw::TabularRawOutputMode};

pub trait OutputMode: std::fmt::Debug {
    fn create(parent: &gtk4::ScrolledWindow) -> Self
    where
        Self: Sized;

    fn format_batches(&self, batches: &[PgResponse]);
}

pub fn create_output_mode(
    parent: &gtk4::ScrolledWindow,
    ty: OutputModeChange,
) -> Option<Box<dyn OutputMode>> {
    Some(match ty {
        OutputModeChange::TabularRaw => Box::new(TabularRawOutputMode::create(parent)),
        OutputModeChange::Csv => Box::new(CsvOutputMode::create(parent)),
        OutputModeChange::Tabular => return None,
    })
}
