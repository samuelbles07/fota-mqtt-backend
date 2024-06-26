use bytes::Bytes;
use reqwest::StatusCode;
use sha2::{Digest, Sha256};
use std::error::Error;

use crate::custom_error::CustomError;
use crate::settings::settings;

#[derive(Debug, Default)]
pub struct BinaryData {
    pub data: Bytes,
    pub hash: Vec<u8>,
    pub current_chunk_id: u16,
    pub last_bytes_index: u16,
}

impl Iterator for BinaryData {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk_size = settings().chunk_size_per_transmission;
        if self.last_bytes_index >= self.data.len() as u16 {
            debug!("No more data of the image");
            return None;
        }

        let until = if self.last_bytes_index + chunk_size < self.data.len() as u16 {
            self.last_bytes_index + chunk_size
        } else {
            // Get the last diff data
            debug!("Last image chunk");
            self.last_bytes_index + (self.data.len() as u16 - self.last_bytes_index)
        };

        let data = self
            .data
            .slice(self.last_bytes_index as usize..until as usize);
        self.last_bytes_index = until;

        // Calculate chunk_id
        let tmp: f32 = self.last_bytes_index as f32 / chunk_size as f32;
        self.current_chunk_id = tmp.ceil() as u16;

        Some(data)
    }
}

pub fn download_binary(url: &String) -> Result<BinaryData, Box<dyn Error>> {
    debug!("Download binary from {url}");
    let body = reqwest::blocking::get(url)?;
    match body.status() {
        StatusCode::OK => {
            let data = body.bytes()?;
            let hash = hash_image(&data);
            Ok(BinaryData {
                data: Bytes::from(data),
                last_bytes_index: 0,
                current_chunk_id: 0,
                hash,
            })
        }
        s => return Err(Box::new(CustomError::HttpRequest(s.as_u16()))),
    }
}

fn hash_image(img: &Bytes) -> Vec<u8> {
    let hash = Sha256::digest(img);
    hash.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_hash_image() {
        let img = Bytes::from("Hello, world!");
        let hash = hash_image(&img);

        assert_eq!(
            hash[..],
            hex!["315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"]
        )
    }

    #[test]
    fn test_hash_image_failed() {
        let img = Bytes::from("Hello, world?");
        let hash = hash_image(&img);

        assert_ne!(
            hash[..],
            hex!["315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"]
        )
    }
}
