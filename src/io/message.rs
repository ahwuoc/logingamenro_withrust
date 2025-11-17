use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
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
    pub fn write_utf(&mut self, value: &str) {
        let bytes = value.as_bytes();
        self.data.put_u16(bytes.len() as u16);
        self.data.put_slice(bytes);
    }
    pub fn read_byte(&mut self) -> Result<i8> {
        Ok(self.data.get_i8())
    }
    pub fn read_int(&mut self) -> Result<i32> {
        Ok(self.data.get_i32())
    }
    pub fn read_long(&mut self) -> Result<i64> {
        Ok(self.data.get_i64())
    }
    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.data.get_u8() != 0)
    }
    pub fn read_utf(&mut self) -> Result<String> {
        let len = self.data.get_u16() as usize;
        let bytes = self.data.split_to(len);
        Ok(String::from_utf8(bytes.to_vec())?)
    }
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}
