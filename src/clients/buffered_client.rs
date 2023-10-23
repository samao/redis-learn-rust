use bytes::Bytes;
use tokio::sync::{
    mpsc::{channel, Receiver, Sender},
    oneshot,
};

use crate::Client;

#[derive(Debug)]
pub struct BufferedClient {
    tx: Sender<Message>,
}

#[derive(Debug)]
enum Command {
    Get(String),
    Set(String, Bytes),
}

type Message = (Command, oneshot::Sender<crate::Result<Option<Bytes>>>);

async fn run(mut client: Client, mut rx: Receiver<Message>) {
    while let Some((cmd, tx)) = rx.recv().await {
        let response = match cmd {
            Command::Get(key) => client.get(&key).await,
            Command::Set(key, value) => client.set(&key, value).await.map(|_| None),
        };

        let _ = tx.send(response);
    }
}

impl BufferedClient {
    pub fn buffer(client: Client) -> BufferedClient {
        let (tx, rx) = channel(32);
        tokio::spawn(async move { run(client, rx).await });

        BufferedClient { tx }
    }

    pub async fn get(&mut self, key: &str) -> crate::Result<Option<Bytes>> {
        let get = Command::Get(key.into());
        let (tx, rx) = oneshot::channel();
        self.tx.send((get, tx)).await?;

        match rx.await {
            Ok(res) => res,
            Err(err) => Err(err.into()),
        }
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> crate::Result<()> {
        let set = Command::Set(key.into(), value);

        let (tx, rx) = oneshot::channel();
        self.tx.send((set, tx)).await?;

        match rx.await {
            Ok(res) => res.map(|_| ()),
            Err(err) => Err(err.into()),
        }
    }
}
