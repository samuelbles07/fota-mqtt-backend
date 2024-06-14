use crate::custom_error::CustomError;
use bytes::Bytes;
use reqwest::StatusCode;
use std::error::Error;

#[derive(Debug)]
pub struct BinaryData {
    pub data: Bytes,
    pub last_bytes_index: u16,
}
// TODO: Implement BinaryData default value
impl Iterator for BinaryData {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: packet size from configuration
        let until = if self.last_bytes_index + 5 < self.data.len() as u16 {
            self.last_bytes_index + 5
        } else {
            self.last_bytes_index + (self.data.len() as u16 - self.last_bytes_index)
        };

        if until >= self.data.len() as u16 {
            self.last_bytes_index = self.data.len() as u16;
            return None;
        }

        let data = self
            .data
            .slice(self.last_bytes_index as usize..until as usize);
        self.last_bytes_index = until;
        Some(data)
    }
}

pub fn download_binary(url: &String) -> Result<BinaryData, Box<dyn Error>> {
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

    // match body.status() {
    //     StatusCode::OK => {
    //         let result = body.bytes()?;
    //         Ok(result)
    //     }
    //     s => Err(Box::new(CustomError::HttpRequest(s.as_u16()))),
    // }
    //
    // let mut mydata = file_handler::BinaryData {
    //     data: Bytes::new(),
    //     last_bytes_index: 0,
    // };
    //
    // match result {
    //     Ok(data) => mydata.data = Bytes::from(data),
    //     Err(err) => {
    //         println!("Error: {err}");
    //         std::process::exit(1);
    //     }
    // };

    // for val in mydata {
    //     println!("{val:?}");
    // }
}
