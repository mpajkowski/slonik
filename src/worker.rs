use anyhow::{anyhow, Result};
use gtk::glib;
use std::{cell::RefCell, pin::Pin};

use futures::{channel::mpsc, StreamExt};
use std::future::Future;

use crate::{components::AppEvent, emit::Emitter};

pub type FutureTask = Pin<Box<dyn Future<Output = ()>>>;

/// Calls futures on `glib::MainContext`
#[derive(Clone)]
pub struct Worker {
    emitter: Emitter,
    sender: RefCell<mpsc::UnboundedSender<FutureTask>>,
}

impl Worker {
    /// spawns `Worker` instance
    pub fn create(ctx: &glib::MainContext, emitter: Emitter) -> Self {
        let (worker_sender, worker_receiver) = mpsc::unbounded::<FutureTask>();

        ctx.spawn_local_with_priority(
            glib::source::PRIORITY_DEFAULT_IDLE,
            worker_receiver.for_each(|f| f),
        );

        Self {
            sender: RefCell::new(worker_sender),
            emitter,
        }
    }

    /// Sends local future of output `()` to `glib::MainContext`.
    ///
    /// # Arguments:
    /// - `task` - future of output `()` with `'static` lifetime
    pub fn send_task<T: Future<Output = ()> + 'static>(&self, task: T) -> Result<()> {
        self.sender
            .borrow_mut()
            .unbounded_send(Box::pin(task))
            .map_err(|_| anyhow!("failed to send task"))
    }

    /// Sends fallible local future of output `anyhow::Result<()>` to `glib::MainContext`.
    ///
    /// If task result is `Err(_)` `AppEvent::Err(e)` should be created.
    ///
    /// # Arguments:
    /// - `task` - future of output `anyhow::Result<()>` with `'static` lifetime
    pub fn send_task_fallible<T: Future<Output = Result<()>> + 'static>(
        &self,
        task: T,
    ) -> Result<()> {
        let emitter = self.emitter.clone();
        self.sender
            .borrow_mut()
            .unbounded_send(Box::pin(async move {
                if let Err(e) = task.await {
                    emitter.emit(AppEvent::Err(e));
                }
            }))
            .map_err(|_| anyhow!("failed to send task"))
    }
}
