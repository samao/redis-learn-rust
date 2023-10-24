use std::{net::SocketAddr, time::Duration};

use mini_redis::server;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

#[tokio::test]
async fn pool_key_value_get_set() {
    let (addr, _) = start_server().await;

    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n")
        .await
        .unwrap();

    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$-1\r\n", &response);

    stream
        .write_all(b"*3\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        .await
        .unwrap();
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);

    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n")
        .await
        .unwrap();
    stream.shutdown().await.unwrap();

    let mut response = [0; 11];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"$5\r\nworld\r\n", &response);
    assert_eq!(0, stream.read(&mut response).await.unwrap());
}

#[tokio::test]
async fn key_value_timeout() {
    tokio::time::pause();
    let (addr, _) = start_server().await;
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(
            b"*5\r\n$3\r\nSET\r\n$5\r\nhello\r\n$5\r\nworld\r\n\
        +EX\r\n:1\r\n",
        )
        .await
        .unwrap();
    let mut response = [0; 5];
    stream.read_exact(&mut response).await.unwrap();
    assert_eq!(b"+OK\r\n", &response);

    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n")
        .await
        .unwrap();
    let mut response = [0; 11];
    stream.read_exact(&mut response).await.unwrap();

    assert_eq!(b"$5\r\nworld\r\n", &response);
    tokio::time::advance(Duration::from_secs(1)).await;
    stream
        .write_all(b"*2\r\n$3\r\nGET\r\n$5\r\nhello\r\n")
        .await
        .unwrap();
    let mut response = [0; 5];

    stream.read_exact(&mut response).await.unwrap();

    assert_eq!(b"$-1\r\n", &response);
}

#[tokio::test]
async fn pub_sub() {
    let (addr, _) = start_server().await;

    let mut publisher = TcpStream::connect(addr).await.unwrap();

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        .await
        .unwrap();

    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":0\r\n", &response);

    let mut sub1 = TcpStream::connect(addr).await.unwrap();
    sub1.write_all(b"*2\r\n$9\r\nSUBSCRIBE\r\n$5\r\nhello\r\n")
        .await
        .unwrap();

    let mut response = [0; 34];
    sub1.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$9\r\nsubscribe\r\n$5\r\nhello\r\n:1\r\n"[..],
        &response[..]
    );

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        .await
        .unwrap();

    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":1\r\n", &response);

    let mut response = [0; 39];
    sub1.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$7\r\nmessage\r\n$5\r\nhello\r\n$5\r\nworld\r\n"[..],
        &response[..]
    );

    let mut sub2 = TcpStream::connect(addr).await.unwrap();
    sub2.write_all(b"*3\r\n$9\r\nSUBSCRIBE\r\n$5\r\nhello\r\n$3\r\nfoo\r\n")
        .await
        .unwrap();

    let mut response = [0; 34];
    sub2.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$9\r\nsubscribe\r\n$5\r\nhello\r\n:1\r\n"[..],
        &response[..]
    );
    let mut response = [0; 32];
    sub2.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$9\r\nsubscribe\r\n$3\r\nfoo\r\n:2\r\n"[..],
        &response[..]
    );

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$5\r\nhello\r\n$5\r\njazzy\r\n")
        .await
        .unwrap();

    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":2\r\n", &response);

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$3\r\nfoo\r\n$3\r\nbar\r\n")
        .await
        .unwrap();

    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":1\r\n", &response);

    let mut response = [0; 39];
    sub1.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$7\r\nmessage\r\n$5\r\nhello\r\n$5\r\njazzy\r\n"[..],
        &response[..]
    );

    let mut response = [0; 39];
    sub2.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$7\r\nmessage\r\n$5\r\nhello\r\n$5\r\njazzy\r\n"[..],
        &response[..]
    );

    let mut response = [0; 1];
    tokio::time::timeout(Duration::from_millis(100), sub1.read(&mut response))
        .await
        .unwrap_err();

    let mut response = [0; 35];
    sub2.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$7\r\nmessage\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"[..],
        &response[..]
    );
}

#[tokio::test]
async fn manage_subscription() {
    let (addr, _) = start_server().await;

    let mut publisher = TcpStream::connect(addr).await.unwrap();

    let mut sub = TcpStream::connect(addr).await.unwrap();
    sub.write_all(b"*2\r\n$9\r\nSUBSCRIBE\r\n$5\r\nhello\r\n")
        .await
        .unwrap();

    let mut response = [0; 34];
    sub.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$9\r\nsubscribe\r\n$5\r\nhello\r\n:1\r\n"[..],
        &response[..]
    );

    sub.write_all(b"*2\r\n$9\r\nSUBSCRIBE\r\n$3\r\nfoo\r\n")
        .await
        .unwrap();

    let mut response = [0; 32];
    sub.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$9\r\nsubscribe\r\n$3\r\nfoo\r\n:2\r\n"[..],
        &response[..]
    );

    sub.write_all(b"*2\r\n$11\r\nUNSUBSCRIBE\r\n$5\r\nhello\r\n")
        .await
        .unwrap();

    let mut response = [0; 37];
    sub.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$11\r\nunsubscribe\r\n$5\r\nhello\r\n:1\r\n"[..],
        &response[..]
    );

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$5\r\nhello\r\n$5\r\nworld\r\n")
        .await
        .unwrap();
    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":0\r\n", &response);

    publisher
        .write_all(b"*3\r\n$7\r\nPUBLISH\r\n$3\r\nfoo\r\n$3\r\nbar\r\n")
        .await
        .unwrap();
    let mut response = [0; 4];
    publisher.read_exact(&mut response).await.unwrap();
    assert_eq!(b":1\r\n", &response);

    let mut response = [0; 35];
    sub.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$7\r\nmessage\r\n$3\r\nfoo\r\n$3\r\nbar\r\n"[..],
        &response[..]
    );

    let mut response = [0; 1];
    tokio::time::timeout(Duration::from_millis(100), sub.read(&mut response))
        .await
        .unwrap_err();

    sub.write_all(b"*1\r\n$11\r\nunsubscribe\r\n")
        .await
        .unwrap();

    let mut response = [0; 35];
    sub.read_exact(&mut response).await.unwrap();
    assert_eq!(
        &b"*3\r\n$11\r\nunsubscribe\r\n$3\r\nfoo\r\n:0\r\n"[..],
        &response[..]
    );
}

async fn start_server() -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move { server::run(listener, tokio::signal::ctrl_c()).await });
    (addr, handle)
}
