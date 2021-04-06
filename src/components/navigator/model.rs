use std::iter;

use crate::{components::AppEvent, emit::Emitter, worker::Worker};
use anyhow::{anyhow, Result};
use url::Url;

pub struct NavigatorModel {
    worker: Worker,
    emitter: Emitter,
}

impl NavigatorModel {
    pub fn new(worker: Worker, emitter: Emitter) -> Self {
        Self { worker, emitter }
    }

    pub fn retrieve_headers(&self, url: &str) -> Result<()> {
        let url = Url::parse(url)?;

        log::debug!("Fetching url: {}", url);

        let emitter = self.emitter.clone();
        self.worker.send_task_fallible(async move {
            let resp = surf::get(url).await.map_err(|err| anyhow!(err))?;
            let headers = resp
                .iter()
                .flat_map(|(name, value)| {
                    iter::repeat_with(move || name.as_str()).zip(value.iter())
                })
                .fold(String::new(), |acc, (name, value)| {
                    format!("{}{}: {}\n", acc, name, value)
                });

            emitter.emit(AppEvent::Contents { contents: headers });

            Ok(())
        })?;

        Ok(())
    }
}
