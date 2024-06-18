use crate::jobs::JobId;
use std::error::Error;
use std::io::Cursor;

#[derive(Debug)]
pub struct Telemetry {
    pub topic: String,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub enum CommandType {
    OtaRequest,
    OtaRequestAck,
    OtaRequestNack,
    OtaDone,
    OtaDoneSuccess,
    OtaDoneFailed,
}

type Command = u8;

impl CommandType {
    fn value(&self) -> Command {
        match *self {
            CommandType::OtaRequest => 0x01,
            CommandType::OtaRequestAck => 0x02,
            CommandType::OtaRequestNack => 0x03,
            CommandType::OtaDone => 0x04,
            CommandType::OtaDoneSuccess => 0x05,
            CommandType::OtaDoneFailed => 0x06,
        }
    }
}

type CommandPayload = (JobId, Command);

pub fn build_command(
    job_id: JobId,
    device_id: &String,
    cmd: CommandType,
) -> Result<Telemetry, Box<dyn Error>> {
    // Format topic
    let topic: String = format!("/fota/cmd/backend/{device_id}");

    // Encode payload to cbor
    let payload: CommandPayload = (job_id, cmd.value());
    let mut buff = Vec::new();
    ciborium::ser::into_writer(&payload, &mut buff)?;

    Ok(Telemetry {
        topic,
        payload: buff,
    })
}

pub fn build_packet(device_id: &String, chunk_id: u16, chunk: bytes::Bytes) -> Telemetry {
    // Format topic
    let topic: String = format!("/fota/data/backend/{device_id}/{chunk_id}");
    Telemetry {
        topic,
        payload: chunk.to_vec(),
    }
}

pub fn parse(tlm: Telemetry) -> Result<CommandPayload, Box<dyn Error>> {
    // let topic_path: Vec<&str> = tlm.topic.split("/").collect();
    // TODO: Define type later either command or chunk. If not for command directly return

    let deserialized: CommandPayload = ciborium::de::from_reader(&mut Cursor::new(tlm.payload))?;
    Ok(deserialized)
}
