use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

// use super::controller::Controller;
use super::message::Message;

pub struct Session {
    pub id: i32,
    pub session_name: String,
    server_id: i32,
    stream: Arc<Mutex<TcpStream>>,
    connected: Arc<AtomicBool>,
    send_key_complete: Arc<AtomicBool>,
    key: Vec<u8>,
    cur_r: u8,
    cur_w: u8,
}

impl Session {
    pub fn new(stream: TcpStream, id: i32) -> Self {
        let session_name = stream
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            id,
            session_name,
            server_id: 0,
            stream: Arc::new(Mutex::new(stream)),
            connected: Arc::new(AtomicBool::new(true)),
            send_key_complete: Arc::new(AtomicBool::new(false)),
            key: b"vmn".to_vec(),
            cur_r: 0,
            cur_w: 0,
        }
    }

    fn read_key(&mut self, b: u8) -> u8 {
        let result = (self.key[self.cur_r as usize] & 0xFF) ^ (b & 0xFF);
        self.cur_r = (self.cur_r + 1) % self.key.len() as u8;
        result
    }

    fn write_key(&mut self, b: u8) -> u8 {
        let result = (self.key[self.cur_w as usize] & 0xFF) ^ (b & 0xFF);
        self.cur_w = (self.cur_w + 1) % self.key.len() as u8;
        result
    }

    pub async fn send_key(&mut self) -> Result<()> {
        if !self.send_key_complete.load(Ordering::Relaxed) {
            let mut msg = Message::new(-27);
            msg.write_byte(self.key.len() as i8);
            msg.write_byte(self.key[0] as i8);

            for i in 1..self.key.len() {
                msg.write_byte((self.key[i] ^ self.key[i - 1]) as i8);
            }
            self.do_send_message(&msg).await?;
            self.send_key_complete.store(true, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn do_send_message(&mut self, msg: &Message) -> Result<()> {
        let data = msg.get_data();
        let value = msg.command;
        let num = data.len();
        let is_encrypted = self.send_key_complete.load(Ordering::Relaxed);

        // STEP 1: Encrypt ALL bytes BEFORE locking stream
        let encrypted_cmd = if is_encrypted {
            self.write_key(value as u8)
        } else {
            value as u8
        };

        let (size_byte1, size_byte2) = if is_encrypted {
            let b1 = self.write_key(((num >> 8) & 0xFF) as u8);
            let b2 = self.write_key((num & 0xFF) as u8);
            (b1, b2)
        } else {
            (0, 0)
        };

        let mut encrypted_data = data.to_vec();
        if is_encrypted {
            for byte in encrypted_data.iter_mut() {
                *byte = self.write_key(*byte);
            }
        }

        // STEP 2: NOW lock stream and write
        let mut stream = self.stream.lock().await;

        stream.write_u8(encrypted_cmd).await?;

        if is_encrypted {
            stream.write_u8(size_byte1).await?;
            stream.write_u8(size_byte2).await?;
        } else {
            stream.write_u16(num as u16).await?;
        }

        stream.write_all(&encrypted_data).await?;
        stream.flush().await?;

        Ok(())
    }

    pub async fn read_message(&mut self) -> Result<Option<Message>> {
        let is_encrypted = self.send_key_complete.load(Ordering::Relaxed);

        let (raw_cmd, raw_size_bytes, size, raw_data) = {
            let mut stream = self.stream.lock().await;

            let cmd = stream.read_u8().await?;

            let (size_b1, size_b2, raw_size) = if is_encrypted {
                let b1 = stream.read_u8().await?;
                let b2 = stream.read_u8().await?;
                (b1, b2, 0)
            } else {
                let sz = stream.read_u16().await? as usize;
                (0, 0, sz)
            };

            let data = if is_encrypted {
                vec![]
            } else {
                let mut buf = vec![0u8; raw_size];
                stream.read_exact(&mut buf).await?;
                buf
            };

            (cmd, (size_b1, size_b2), raw_size, data)
        }; // stream guard dropped here

        let cmd = if is_encrypted {
            let decrypted = self.read_key(raw_cmd);
            decrypted
        } else {
            raw_cmd
        };

        // Decrypt size and read data
        let mut data = if is_encrypted {
            // Decrypt size bytes to get actual data length
            let b1 = self.read_key(raw_size_bytes.0);
            let b2 = self.read_key(raw_size_bytes.1);
            let actual_size = ((b1 as usize) << 8) | (b2 as usize);
            println!(
                "DEBUG: Decrypted size: {} -> {}",
                ((raw_size_bytes.0 as usize) << 8) | (raw_size_bytes.1 as usize),
                actual_size
            );

            // Now read the actual data with correct size
            let mut stream = self.stream.lock().await;
            let mut buf = vec![0u8; actual_size];
            stream.read_exact(&mut buf).await?;
            drop(stream);

            // Decrypt data
            for byte in buf.iter_mut() {
                *byte = self.read_key(*byte);
            }

            buf
        } else {
            raw_data
        };

        Ok(Some(Message::with_data(cmd as i8, data)))
    }

    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
    pub async fn send_message(&mut self, msg: &Message) -> Result<()> {
        self.do_send_message(msg).await
    }

    pub fn is_key_sent(&self) -> bool {
        self.send_key_complete.load(Ordering::Relaxed)
    }

    pub fn close(&self) {
        self.connected.store(false, Ordering::Relaxed);
    }
}
