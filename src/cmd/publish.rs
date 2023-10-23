use bytes::Bytes;

use crate::{db::Db, parse::Parse, Connection, Frame};

#[derive(Debug)]
pub struct Publish {
    channel: String,
    message: Bytes,
}

impl Publish {
    pub(crate) fn new(channel: impl ToString, message: Bytes) -> Publish {
        Publish {
            channel: channel.to_string(),
            message,
        }
    }

    pub(crate) fn parse_frames(parse: &mut Parse) -> crate::Result<Publish> {
        let channel = parse.next_string()?;
        let message = parse.next_bytes()?;
        Ok(Publish { channel, message })
    }

    pub(crate) async fn apply(self, db: &Db, dst: &mut Connection) -> crate::Result<()> {
        let num_subscribers = db.publish(&self.channel, self.message);
        let response = Frame::Integer(num_subscribers as u64);

        dst.write_frame(&response).await?;

        Ok(())
    }

    pub(crate) fn into_frame(self) -> Frame {
        let mut frame = Frame::array();
        frame.push_bulk(Bytes::from("publish".as_bytes()));
        frame.push_bulk(Bytes::from(self.channel.into_bytes()));
        frame.push_bulk(self.message);
        frame
    }
}
