use csv::ByteRecord;

use super::{output_mode::OutputMode, textutils::format_text};
use crate::model::pg_response::{PgResponse, Table};

#[derive(Debug)]
pub struct CsvOutputMode {
    widget: gtk4::TextView,
}

impl OutputMode for CsvOutputMode {
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

    fn format_batches(&self, batches: &[PgResponse]) {
        format_text(&self.widget, batches, format_csv);
    }
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
