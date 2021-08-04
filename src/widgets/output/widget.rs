use std::{sync::Arc, time::Instant};

use gtk4::prelude::*;
use tokio_postgres::error::DbError;

use crate::{
    event::{AppEvent, Emitter, EventListener, OutputModeChange},
    model::pg_response::PgResponse,
    object_or_expect,
};

use super::output_mode::{create_output_mode, OutputMode};

pub struct Output {
    output_mode: Box<dyn OutputMode>,
    output_buffer: gtk4::ScrolledWindow,
    batches: Arc<Vec<PgResponse>>,
}

impl Output {
    pub fn create(builder: &gtk4::Builder, emitter: Emitter) -> Self {
        let output_buffer: gtk4::ScrolledWindow = object_or_expect(builder, "output");

        let radio_tabular: gtk4::CheckButton = object_or_expect(builder, "output-mode-tabular");
        let radio_csv: gtk4::CheckButton = object_or_expect(builder, "output-mode-csv");
        let radio_tabular_raw: gtk4::CheckButton =
            object_or_expect(builder, "output-mode-tabular-raw");

        let emitter_c = emitter.clone();
        radio_tabular.connect_toggled(move |btn| {
            if btn.is_active() {
                emitter_c.emit(AppEvent::OutputModeChanged(OutputModeChange::Tabular))
            }
        });

        let emitter_c = emitter.clone();
        radio_csv.connect_toggled(move |btn| {
            if btn.is_active() {
                emitter_c.emit(AppEvent::OutputModeChanged(OutputModeChange::Csv))
            }
        });

        radio_tabular_raw.connect_toggled(move |btn| {
            if btn.is_active() {
                emitter.emit(AppEvent::OutputModeChanged(OutputModeChange::TabularRaw))
            }
        });

        let output_mode = create_output_mode(&output_buffer, OutputModeChange::Tabular);

        Self {
            output_buffer,
            output_mode,
            batches: Arc::new(vec![]),
        }
    }

    fn on_pg_response(&mut self, responses: Arc<Vec<PgResponse>>) {
        self.batches = responses;
        self.format_batches();
    }

    fn on_db_err(&self, err: &DbError) {
        let dialog = gtk4::MessageDialogBuilder::new()
            .modal(true)
            .title("Database error")
            .text(err.message())
            .buttons(gtk4::ButtonsType::Close)
            .build();

        dialog.show();
    }

    fn on_output_mode_changed(&mut self, ty: OutputModeChange) {
        self.output_mode = create_output_mode(&self.output_buffer, ty);
        self.format_batches();
    }
}

impl Output {
    fn format_batches(&self) {
        let instant = Instant::now();
        self.output_mode.format_batches(&self.batches);
        log::info!("Formatting batches took {:?}", instant.elapsed());
    }
}

impl EventListener for Output {
    fn on_event(&mut self, event: &AppEvent) {
        use tokio_postgres::Error as PgError;

        match event {
            AppEvent::PgResponses { id: _, responses } => {
                self.on_pg_response(Arc::clone(responses))
            }
            AppEvent::Err(err) => {
                let db_err = match err.downcast_ref::<PgError>().and_then(PgError::as_db_error) {
                    Some(err) => err,
                    None => return,
                };

                self.on_db_err(db_err);
            }
            AppEvent::OutputModeChanged(ty) => {
                self.on_output_mode_changed(*ty);
            }
            _ => {}
        }
    }
}
