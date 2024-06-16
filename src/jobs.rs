use crate::file_handler::BinaryData;
use bytes::Bytes;
use rand::Rng;
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

    pub fn add_job(&mut self, device_id: String, url: String) {
        let job_id = Self::generate_job_id();
        self.jobs.insert(
            job_id,
            Job {
                job_id,
                device_id,
                status: JobStatus::OnQueue,
                url,
                image: BinaryData::default(),
            },
        );

        println!("added job {:#?}", self.jobs.get(&123));
    }

    fn generate_job_id() -> u16 {
        let mut rng = rand::thread_rng();
        rng.gen_range(1000..=9999)
    }
}
