use crate::file_handler::BinaryData;
use bytes::Bytes;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
enum JobStatus {
    Success,
    Failed,
    InProgress,
    OnQueue,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Job {
    job_id: u16,
    device_id: String,
    status: JobStatus,
    url: String,
    image: BinaryData,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct JobScheduler {
    jobs: HashMap<u16, Job>,
    running: Vec<u16>,
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            running: Vec::new(),
        }
    }

    // TODO: jobid generator
    pub fn add_job(&mut self, device_id: String, url: String) {
        self.jobs.insert(
            123,
            Job {
                job_id: 123,
                device_id,
                status: JobStatus::OnQueue,
                url,
                image: BinaryData {
                    data: Bytes::new(),
                    last_bytes_index: 0,
                },
            },
        );

        println!("added job {:#?}", self.jobs.get(&123));
    }
}
