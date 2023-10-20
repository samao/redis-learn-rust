/*
 * Copyright (c) QieTv, Inc. 2018
 * @Author: idzeir
 * @Date: 2023-10-20 12:43:58
 * @Last Modified by: idzeir
 * @Last Modified time: 2023-10-20 12:49:31
 */

use bytes::Bytes;

use crate::Connection;

pub struct Client {
    connection: Connection,
}

pub struct Subscriber {
    client: Client,
    subscribed_channels: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub channel: String,
    pub content: Bytes,
}
