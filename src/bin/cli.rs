use mini_redis::{clients::Client, DEFAULT_PORT};

use bytes::Bytes;
use clap::{Parser, Subcommand};
use std::convert::Infallible;
use std::num::ParseIntError;
use std::time::Duration;
use tracing::info;

#[derive(Debug, Parser)]
#[clap(
    name = "mini-redis-cli",
    version,
    author,
    about = "issue redis commands"
)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

#[derive(Debug, Subcommand)]
enum Command {
    Ping {
        #[clap(value_parser = bytes_from_str)]
        msg: Option<Bytes>,
    },
    Get {
        key: String,
    },
    Set {
        key: String,
        #[clap(value_parser = bytes_from_str)]
        value: Bytes,
        #[clap(value_parser= duration_from_ms_str)]
        expires: Option<Duration>,
    },
    Publish {
        channel: String,

        #[clap(value_parser=bytes_from_str)]
        message: Bytes,
    },
    Subscribe {
        channels: Vec<String>,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> mini_redis::Result<()> {
    info!("hello client");
    tracing_subscriber::fmt::try_init()?;
    let cli = Cli::parse();
    let addr = format!("{}:{}", cli.host, cli.port);

    let mut client = Client::connect(&addr).await?;
    match cli.command {
        Command::Ping { msg } => {
            let value = client.ping(msg).await?;
            if let Ok(string) = String::from_utf8(value.to_vec()) {
                println!("\"{}\"", string);
            } else {
                println!("{:?}", value);
            }
        }
        Command::Get { key } => {
            if let Some(value) = client.get(&key).await? {
                if let Ok(string) = String::from_utf8(value.to_vec()) {
                    println!("\"{}\"", string);
                } else {
                    println!("{:?}", value);
                }
            }
        }
        Command::Set {
            key,
            value,
            expires: None,
        } => {
            client.set(&key, value).await?;
            println!("OK");
        }
        Command::Set {
            key,
            value,
            expires: Some(expires),
        } => {
            client.set_expirse(&key, value, expires).await?;
            println!("OK");
        }
        Command::Publish { channel, message } => {
            client.publish(&channel, message).await?;
            println!("Publish OK");
        }
        Command::Subscribe { channels } => {
            if channels.is_empty() {
                return Err("channel(s) must be provided".into());
            }
            let mut subscriber = client.subscribe(channels).await?;

            while let Some(msg) = subscriber.next_message().await? {
                println!(
                    "got message from the channel: {}; message = {:?}",
                    msg.channel, msg.content
                );
            }
        }
    }
    Ok(())
}

fn duration_from_ms_str(src: &str) -> Result<Duration, ParseIntError> {
    let ms = src.parse::<u64>()?;
    Ok(Duration::from_millis(ms))
}

fn bytes_from_str(src: &str) -> Result<Bytes, Infallible> {
    Ok(Bytes::from(src.to_string()))
}
