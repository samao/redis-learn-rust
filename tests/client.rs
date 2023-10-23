use std::net::SocketAddr;

use mini_redis::{server, Client};
use tokio::{net::TcpListener, task::JoinHandle};

#[tokio::test]
async fn ping_pong_without_message() {
    let (addr, _) = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();
    let pong = client.ping(None).await.unwrap();
    assert_eq!(b"PONG", &pong[..]);
}

#[tokio::test]
async fn ping_pong_with_message() {
    let (addr, _) = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();
    let pong = client.ping(Some("你好世界".into())).await.unwrap();
    assert_eq!("你好世界".as_bytes(), &pong[..]);
}

#[tokio::test]
async fn key_value_get_set() {
    let (addr, _) = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();
    client.set("hello", "不好".into()).await.unwrap();

    let value = client.get("hello").await.unwrap().unwrap();
    assert_eq!("不好".as_bytes(), &value[..]);
}

#[tokio::test]
async fn receive_message_subscribed_channel() {
    let (addr, _) = start_server().await;
    let client = Client::connect(addr).await.unwrap();
    let mut subscriber = client.subscribe(vec!["hello".into()]).await.unwrap();
    tokio::spawn(async move {
        let mut client = Client::connect(addr).await.unwrap();
        client.publish("hello", "world".into()).await.unwrap();
    });
    let message = subscriber.next_message().await.unwrap().unwrap();
    assert_eq!("hello", &message.channel);
    assert_eq!(b"world", &message.content[..]);
}

#[tokio::test]
async fn receive_message_multiple_subscribed_channels() {
    let (addr, _) = start_server().await;
    let client = Client::connect(addr).await.unwrap();
    let mut subscriber = client
        .subscribe(vec!["hello".into(), "world".into()])
        .await
        .unwrap();
    tokio::spawn(async move {
        let mut client = Client::connect(addr).await.unwrap();
        client.publish("hello", "world".into()).await.unwrap();
    });

    let message1 = subscriber.next_message().await.unwrap().unwrap();
    assert_eq!("hello", &message1.channel);
    assert_eq!(b"world", &message1.content[..]);
    tokio::spawn(async move {
        let mut client = Client::connect(addr).await.unwrap();
        client.publish("world", "howdy?".into()).await.unwrap();
    });
    let message2 = subscriber.next_message().await.unwrap().unwrap();
    assert_eq!("world", &message2.channel);
    assert_eq!(b"howdy?", &message2.content[..]);
}

#[tokio::test]
async fn unsubscribes_from_channels() {
    let (addr, _) = start_server().await;
    let client = Client::connect(addr).await.unwrap();
    let mut subscriber = client
        .subscribe(vec!["hello".into(), "world".into()])
        .await
        .unwrap();

    subscriber.unsubscribe(&[]).await.unwrap();

    assert_eq!(subscriber.get_subscribed().len(), 0);
}

async fn start_server() -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    (addr, handle)
}
