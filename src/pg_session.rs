use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::env;

use anyhow::Result;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::{stream, StreamExt, TryStreamExt};
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
    fn on_event(&mut self, event: &AppEvent) {
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
                let session = PgSession::initialize(emitter.clone()).await?;
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
}

impl PgSession {
    async fn initialize(emitter: Emitter) -> Result<Self> {
        let mut cfg = Config::new();
        cfg.host(env::var("PG_HOST").as_deref().unwrap_or("localhost"));
        cfg.port(env::var("PG_PORT").as_deref().unwrap_or("5432").parse()?);
        cfg.user(env::var("PG_USER").as_deref().unwrap_or("postgres"));
        if let Ok(pg_pass) = env::var("PG_PASSWORD") {
            cfg.password(&pg_pass);
        }

        let (client, mut conn) = cfg.connect(NoTls).await?;
        let (tx, mut rx) = unbounded::<AsyncMessage>();

        let stream = stream::poll_fn(move |cx| conn.poll_message(cx)).map_err(|e| panic!("{}", e));
        let connection = stream.forward(tx);

        tokio::spawn(connection);
        tokio::spawn(async move {
            while let Some(async_msg) = rx.next().await {
                emitter.emit(AppEvent::PgMessage(async_msg))
            }
        });

        Ok(Self { client })
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
