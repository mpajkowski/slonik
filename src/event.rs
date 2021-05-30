use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    StreamExt,
};

use crate::model::pg_response::PgResponse;

#[derive(Debug)]
#[non_exhaustive]
pub enum AppEvent {
    Started,
    PgRequest(PgRequest),
    PgResponses(Vec<PgResponse>),
    Err(anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PgRequest {
    pub id: usize,
    pub text: String,
}

pub trait EventListener {
    fn on_event(&self, event: &AppEvent);
}

/// Sender which emits particular `AppEvents`
#[derive(Clone)]
pub struct Emitter {
    sender: UnboundedSender<AppEvent>,
}

impl Emitter {
    fn new(sender: UnboundedSender<AppEvent>) -> Self {
        Self { sender }
    }

    pub fn emit(&self, event: AppEvent) {
        self.sender.unbounded_send(event).unwrap();
    }
}

/// Dispatches events to subscribers
pub struct DispatchLoop {
    sender: UnboundedSender<AppEvent>,
    receiver: UnboundedReceiver<AppEvent>,
    listeners: Vec<Box<dyn EventListener>>,
}

impl DispatchLoop {
    pub fn create() -> Self {
        let (sender, receiver) = mpsc::unbounded();
        Self {
            sender,
            receiver,
            listeners: vec![],
        }
    }

    pub fn register_listener<T: EventListener + 'static>(&mut self, listener: T) {
        self.listeners.push(Box::new(listener));
    }

    pub fn create_emitter(&self) -> Emitter {
        Emitter::new(self.sender.clone())
    }

    pub async fn listen(mut self) {
        while let Some(event) = self.receiver.next().await {
            self.listeners
                .iter()
                .for_each(|listener| listener.on_event(&event))
        }
    }
}
