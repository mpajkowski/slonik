use gtk::{TextBufferExt, TextViewExt};
use itertools::Itertools;

use crate::{
    event::{AppEvent, EventListener},
    model::pg_response::PgResponse,
    object_or_expect,
};

pub struct Output {
    widget: gtk::TextView,
}

impl Output {
    pub fn create(builder: &gtk::Builder) -> Self {
        let widget: gtk::TextView = object_or_expect(builder, "output");

        Self { widget }
    }
}

impl EventListener for Output {
    fn on_event(&self, event: &AppEvent) {
        use tokio_postgres::Error as PgError;

        match event {
            AppEvent::PgResponses(pg_responses) => {
                let buf = self.widget.get_buffer().unwrap();

                let new_buffer = pg_responses
                    .iter()
                    .map(|response| match response {
                        PgResponse::Table(table) => table.as_pretty().to_string(),
                        PgResponse::CommandComplete(rows_affected) => {
                            format!("rows affected: {}", rows_affected)
                        }
                    })
                    .join("\n");

                buf.set_text(&new_buffer);
            }
            AppEvent::Err(err) => {
                println!("ERR1: :{}", err);
                let db_err = match err.downcast_ref::<PgError>().and_then(PgError::as_db_error) {
                    Some(err) => err,
                    None => return,
                };

                let buf = self.widget.get_buffer().unwrap();
                buf.set_text(&db_err.to_string());
            }
            _ => {}
        }
    }
}
