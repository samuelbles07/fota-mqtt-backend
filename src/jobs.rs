use crate::custom_error::CustomError;
use crate::file_handler::{download_binary, BinaryData};
use bytes::Bytes;
use core::time;
use rand::Rng;
use reqwest::StatusCode;
use std::error::Error;
use std::{
    collections::{HashMap, VecDeque},
    thread,
};

const MAX_RUNNING_JOB: usize = 3;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
enum JobStatus {
    Success,
    Failed,
    InProgress,
    OnQueue,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Job {
    job_id: u16, // TODO: Change type to jobId own type
    device_id: String,
    status: JobStatus,
    url: String,
    image: BinaryData,
}

#[allow(dead_code)]
pub struct JobScheduler {
    jobs: HashMap<u16, Job>,
    on_queue: VecDeque<u16>,
    running: Vec<u16>,
    last_running_job_index: u8,
}

impl JobScheduler {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            running: Vec::new(),
            on_queue: VecDeque::new(),
            last_running_job_index: 0,
        }
    }

    pub fn run(&mut self) {
        // Just to simulate or testing purposes
        self.add_job(
            "device1".to_string(),
            "http://localhost:7777/bin/tes.txt".to_string(),
        );

        self.add_job(
            "device2".to_string(),
            "http://localhost:7777/bin/test2.txt".to_string(),
        );

        self.add_job(
            "device3".to_string(),
            "http://localhost:7777/bin/test3.txt".to_string(),
        );

        loop {
            // TODO: Here receive new job

            // Start job from on_queue job list if running list not in max number
            if self.running.len() < MAX_RUNNING_JOB {
                if let Err(msg) = self.start_job_onqueue() {
                    println!("{msg}");
                    // TODO: do somekind of timeout for checking this if statement
                }
            }

            // Give cpu/thread some breath when no job in running status
            let Some(next_job_id) = self.get_next_job() else {
                println!("No running job exists");
                thread::sleep(time::Duration::from_secs(1));
                // thread::sleep(time::Duration::from_millis(100));
                continue;
            };

            println!("Next Job: {next_job_id}");
            self.process_job(next_job_id);

            thread::sleep(time::Duration::from_secs(1));
        }
    }

    fn add_job(&mut self, device_id: String, url: String) {
        // Add new job to the on_queue list
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

        self.on_queue.push_back(job_id);
        println!("added job {:#?}", self.jobs.get(&job_id));
    }

    fn start_job_onqueue(&mut self) -> Result<(), CustomError> {
        // Get job that still on queue.
        // Only get reference since, needs to process it first before removing it from the queue
        let Some(job_id) = self.on_queue.front() else {
            return Err(CustomError::StartJob(String::from(
                "No job that still OnQueue",
            )));
        };

        // Try to get job data (as mutable reference) to be modified later
        let Some(job) = self.jobs.get_mut(job_id) else {
            return Err(CustomError::StartJob(format!(
                "No job in hashmap with id {}",
                job_id
            )));
        };

        // Attempt to download the binary from url provided
        match download_binary(&job.url) {
            Ok(data) => {
                // Now the binary already on the heap (BinaryData) and ready to chunked
                job.image = data;
                // Add the job to running index list
                self.running.push(job_id.clone());
                // Remove the job from on_queue index list
                _ = self.on_queue.remove(0);
                // Change the actual job data status to in progres
                job.status = JobStatus::InProgress;
                Ok(())
            }
            Err(msg) => {
                // TODO: Remove from queue?
                println!("{msg}");
                Err(CustomError::StartJob(String::from("Failed download")))
            }
        }
    }

    fn process_job(&mut self, job_id: u16) {
        let Some(job) = self.jobs.get_mut(&job_id) else {
            // TODO: Return custom error
            // return Err(CustomError::StartJob(format!(
            //     "No job in hashmap with id {}",
            //     job_id
            // )));
            return;
        };

        match job.image.next() {
            Some(chunk) => println!("data of {} => {:?}", job_id, chunk),
            None => {
                println!("Job {job_id} finish");
                // remove job from running list and change job status on hashmap
                job.status = JobStatus::Success;
                // job.status = JobStatus::Failed;
                self.running.remove(self.last_running_job_index.into());
            }
        }
    }

    fn get_next_job(&mut self) -> Option<u16> {
        // Return directly when running job list is empty
        if self.running.len() == 0 {
            return None;
        }

        // Attempt to get next job from the running job index
        // If already on last index, then start over
        let idx: u8 = self.last_running_job_index + 1;
        let Some(value) = self.running.get(idx as usize) else {
            // println!("Already on last index");
            self.last_running_job_index = 0;
            return self.running.get(0).copied();
        };

        // Use the next index, keep the index to last variable
        self.last_running_job_index = idx;
        Some(value.clone())
    }

    fn generate_job_id() -> u16 {
        let mut rng = rand::thread_rng();
        rng.gen_range(1000..=9999)
    }
}
