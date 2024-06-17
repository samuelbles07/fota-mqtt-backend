use crate::custom_error::CustomError;
use bytes::Bytes;
use reqwest::StatusCode;
use std::error::Error;

const CHUNK_SIZE: u16 = 5;

#[derive(Debug, Default)]
pub struct BinaryData {
    pub data: Bytes,
    pub last_bytes_index: u16,
}

impl Iterator for BinaryData {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_bytes_index >= self.data.len() as u16 {
            // No more data
            return None;
        }

        let until = if self.last_bytes_index + CHUNK_SIZE < self.data.len() as u16 {
            self.last_bytes_index + CHUNK_SIZE
        } else {
            // Get the last diff data
            self.last_bytes_index + (self.data.len() as u16 - self.last_bytes_index)
        };

        let data = self
            .data
            .slice(self.last_bytes_index as usize..until as usize);
        self.last_bytes_index = until;
        Some(data)
    }
}

pub fn download_binary(url: &String) -> Result<BinaryData, Box<dyn Error>> {
    println!("Download binary from {url}");
    let body = reqwest::blocking::get(url)?;
    match body.status() {
        StatusCode::OK => {
            let data = body.bytes()?;
            Ok(BinaryData {
                data: Bytes::from(data),
                last_bytes_index: 0,
            })
        }
        s => return Err(Box::new(CustomError::HttpRequest(s.as_u16()))),
    }
}
