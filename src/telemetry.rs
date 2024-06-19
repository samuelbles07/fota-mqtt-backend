use crate::jobs::JobId;
use std::error::Error;
use std::io::Cursor;

#[derive(Debug)]
pub struct Telemetry {
    pub topic: String,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum CommandType {
    OtaRequest = 0x01,
    OtaRequestAck,
    OtaRequestNack,
    OtaDone,
    OtaDoneSuccess,
    OtaDoneFailed,
}

impl From<u8> for CommandType {
    fn from(value: u8) -> Self {
        match value {
            0x01 => Self::OtaRequest,
            0x02 => Self::OtaRequestAck,
            0x03 => Self::OtaRequestNack,
            0x04 => Self::OtaDone,
            0x05 => Self::OtaDoneSuccess,
            0x06 => Self::OtaDoneFailed,
        }
    }
}

type CommandPayload = (JobId, u8); // u8 is CommandType as u8

pub fn build_command(
    job_id: JobId,
    device_id: &String,
    cmd: CommandType,
) -> Result<Telemetry, Box<dyn Error>> {
    // Format topic
    let topic: String = format!("/fota/cmd/{device_id}");

    // Encode payload to cbor
    let payload: CommandPayload = (job_id, cmd as u8);
    let mut buff = Vec::new();
    ciborium::ser::into_writer(&payload, &mut buff)?;

    Ok(Telemetry {
        topic,
        payload: buff,
    })
}

pub fn build_packet(device_id: &String, chunk_id: u16, chunk: bytes::Bytes) -> Telemetry {
    // Format topic
    let topic: String = format!("/fota/data/{device_id}/{chunk_id}");
    Telemetry {
        topic,
        payload: chunk.to_vec(),
    }
}

pub fn parse(tlm: Telemetry) -> Result<(JobId, CommandType), Box<dyn Error>> {
    // let topic_path: Vec<&str> = tlm.topic.split("/").collect();
    // TODO: Define type later either command or chunk. If not for command directly return

    let deserialized: CommandPayload = ciborium::de::from_reader(&mut Cursor::new(tlm.payload))?;
    Ok((deserialized.0, CommandType::from(deserialized.1)))
}
