use std::collections::hash_map::Entry;
use std::collections::HashMap;

use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::{stream, FutureExt, StreamExt, TryStreamExt};
use tokio_postgres::SimpleQueryMessage;
use tokio_postgres::{AsyncMessage, Client, Config, NoTls};

use crate::event::PgRequest;
use crate::event::{AppEvent, Emitter, EventListener};
use crate::model::pg_response::PgResponse;

pub struct PgEventLoopProxy {
    event_loop_tx: UnboundedSender<PgRequest>,
}

impl PgEventLoopProxy {
    pub fn initialize(emitter: Emitter) -> Self {
        let (event_loop_tx, event_loop_rx) = unbounded();

        tokio::spawn(async move { pg_event_loop(emitter, event_loop_rx).await.unwrap() });

        Self { event_loop_tx }
    }
}

impl EventListener for PgEventLoopProxy {
    fn on_event(&self, event: &AppEvent) {
        if let AppEvent::PgRequest(req) = event {
            self.event_loop_tx.unbounded_send(req.clone()).unwrap();
        }
    }
}

pub async fn pg_event_loop(
    emitter: Emitter,
    mut receiver: UnboundedReceiver<PgRequest>,
) -> Result<()> {
    let mut sessions: HashMap<usize, PgSession> = HashMap::new();

    while let Some(PgRequest { id, text }) = receiver.next().await {
        let session = match sessions.entry(id) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => {
                let session = PgSession::initialize().await?;
                v.insert(session)
            }
        };

        emitter.emit(match session.exec_simple_query(&text).await {
            Ok(batches) => AppEvent::PgResponses(PgResponse::process_batches(batches)),
            Err(err) => AppEvent::Err(err),
        })
    }

    Ok(())
}

pub struct PgSession {
    client: Client,
    rx: UnboundedReceiver<AsyncMessage>,
}

impl PgSession {
    async fn initialize() -> Result<Self> {
        let mut cfg = Config::new();
        cfg.user("postgres");
        cfg.host("localhost");
        cfg.port(5432);

        let (client, mut conn) = cfg.connect(NoTls).await?;
        let (tx, rx) = unbounded::<AsyncMessage>();

        let stream = stream::poll_fn(move |cx| conn.poll_message(cx)).map_err(|e| panic!("{}", e));
        let connection = stream.forward(tx).map(|r| r.unwrap());

        tokio::spawn(connection);

        Ok(Self { client, rx })
    }

    async fn exec_simple_query(&mut self, text: &str) -> Result<Vec<SimpleQueryMessage>> {
        let tx = self.client.transaction().await?;
        let batches = tx.simple_query(&*text).await?;

        /*
        for msg in &batches {
            match msg {
                SimpleQueryMessage::Row(row) => {
                    for i in 0..row.len() {
                        print!("{}, ", row.columns()[i].name());
                    }
                    println!();
                    for i in 0..row.len() {
                        print!("{}, ", row.get(i).unwrap_or("[null]"));
                    }
                    println!();
                }
                SimpleQueryMessage::CommandComplete(c) => println!("CC: {}", c),
                _ => {}
            }
        }
        */

        tx.commit().await?;

        Ok(batches)
    }
}
