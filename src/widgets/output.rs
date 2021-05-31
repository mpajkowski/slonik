use csv::ByteRecord;
use gtk::prelude::*;
use itertools::Itertools;

use crate::{
    event::{AppEvent, Emitter, EventListener},
    model::pg_response::{PgResponse, Table},
    object_or_expect,
};

pub struct Output {
    widget: gtk::TextView,
    output_mode: OutputMode,
    batches: Vec<PgResponse>,
}

impl Output {
    pub fn create(builder: &gtk::Builder, emitter: Emitter) -> Self {
        let widget: gtk::TextView = object_or_expect(builder, "output");

        let radio_tabular: gtk::RadioButton = object_or_expect(builder, "output-mode-tabular");
        let radio_csv: gtk::RadioButton = object_or_expect(builder, "output-mode-csv");
        let radio_tabular_raw: gtk::RadioButton =
            object_or_expect(builder, "output-mode-tabular-raw");

        let emitter_c = emitter.clone();
        radio_tabular.connect_toggled(move |btn| {
            if btn.get_active() {
                emitter_c.emit(AppEvent::OutputModeChanged(OutputMode::Tabular))
            }
        });

        let emitter_c = emitter.clone();
        radio_csv.connect_toggled(move |btn| {
            if btn.get_active() {
                emitter_c.emit(AppEvent::OutputModeChanged(OutputMode::Csv))
            }
        });

        radio_tabular_raw.connect_toggled(move |btn| {
            if btn.get_active() {
                emitter.emit(AppEvent::OutputModeChanged(OutputMode::TabularRaw))
            }
        });

        Self {
            widget,
            output_mode: OutputMode::Tabular,
            batches: vec![],
        }
    }
}

impl Output {
    fn redraw(&self) {
        let buf = self.widget.get_buffer().unwrap();
        buf.set_text(&self.format_batches())
    }

    fn format_batches(&self) -> String {
        self.batches
            .iter()
            .map(|batch| match batch {
                PgResponse::Table(table) => self.format_table(&table),
                PgResponse::CommandComplete(cc) => format!("rows_affected: {}", cc),
            })
            .join("\n")
    }

    fn format_table(&self, table: &Table) -> String {
        match self.output_mode {
            OutputMode::Tabular => Self::format_raw(table),
            OutputMode::Csv => Self::format_csv(table),
            OutputMode::TabularRaw => Self::format_raw(table),
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

    fn format_csv(table: &Table) -> String {
        let output = vec![];
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_writer(output);

        let header_record = ByteRecord::from(table.header.columns.as_slice());
        writer.write_byte_record(&header_record).unwrap();

        for row in table.rows.iter() {
            let sanitized_row = row
                .values
                .iter()
                .map(|x| x.as_deref().map(|x| x.as_bytes()).unwrap_or_default())
                .collect::<Vec<_>>();

            let record = ByteRecord::from(sanitized_row.as_slice());
            writer.write_byte_record(&record).unwrap();
        }

        String::from_utf8(writer.into_inner().unwrap()).unwrap()
    }
}

impl EventListener for Output {
    fn on_event(&mut self, event: &AppEvent) {
        use tokio_postgres::Error as PgError;

        match event {
            AppEvent::PgResponses(pg_responses) => {
                self.batches = pg_responses.clone();
                self.redraw();
            }
            AppEvent::Err(err) => {
                let db_err = match err.downcast_ref::<PgError>().and_then(PgError::as_db_error) {
                    Some(err) => err,
                    None => return,
                };

                let buf = self.widget.get_buffer().unwrap();
                buf.set_text(&db_err.to_string());
            }
            AppEvent::OutputModeChanged(output_mode) => {
                self.output_mode = *output_mode;
                self.redraw();
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Tabular,
    Csv,
    TabularRaw,
}
