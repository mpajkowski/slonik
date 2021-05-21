use anyhow::{anyhow, Result};
use std::{cell::RefCell, pin::Pin, sync::Arc};
use tokio::runtime::Runtime;

use futures::{channel::mpsc, StreamExt, TryFutureExt};
use std::future::Future;

use crate::{components::AppEvent, emit::Emitter};

pub type FutureTask = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Calls futures on tokio's `Runtime`
#[derive(Clone)]
pub struct Worker {
    runtime: Arc<Runtime>,
    emitter: Emitter,
}

impl Worker {
    /// spawns `Worker` instance
    pub fn create(emitter: Emitter) -> Result<Self> {
        let (worker_sender, worker_receiver) = mpsc::unbounded::<FutureTask>();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            emitter,
            runtime: Arc::new(rt),
        })
    }

    /// Sends local future of output `()` to `tokio::Runtime`.
    ///
    /// # Arguments:
    /// - `task` - future of output `()` with `'static` lifetime
    pub fn send_task<T: Future<Output = ()> + Send + 'static>(&self, task: T) {
        self.runtime.spawn(task);
    }

    /// Sends fallible local future of output `anyhow::Result<()>` to `glib::MainContext`.
    ///
    /// If task result is `Err(_)` `AppEvent::Err(e)` should be created.
    ///
    /// # Arguments:
    /// - `task` - future of output `anyhow::Result<()>` with `'static` lifetime
    pub fn send_task_fallible<T: Future<Output = Result<()>> + Send + 'static>(&self, task: T) {
        let emitter = self.emitter.clone();

        self.runtime.spawn(async move {
            if let Err(e) = task.await {
                emitter.emit(AppEvent::Err(e));
            }
        });
    }
}
