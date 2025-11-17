use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use std::io::Cursor;

pub struct Message {
    pub command: i8,
    data: BytesMut,
}

impl Message {
    pub fn new(command: i8) -> Self {
        Self {
            command,
            data: BytesMut::new(),
        }
    }
    pub fn with_data(command: i8, data: Vec<u8>) -> Self {
        Self {
            command,
            data: BytesMut::from(&data[..]),
        }
    }
    pub fn write_byte(&mut self, value: i8) {
        self.data.put_i8(value);
    }
    pub fn write_int(&mut self, value: i32) {
        self.data.put_i32(value);
    }
    pub fn write_long(&mut self, value: i64) {
        self.data.put_i64(value);
    }
    pub fn write_bool(&mut self, value: bool) {
        self.data.put_u8(if value { 1 } else { 0 });
    }
}
