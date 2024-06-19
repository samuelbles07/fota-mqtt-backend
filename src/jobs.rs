use crate::custom_error::CustomError;
use crate::file_handler::{download_binary, BinaryData};
use crate::messenger::Messenger;
use crate::telemetry::{self, CommandType, Telemetry};
// use bytes::Bytes;
use core::time;
use rand::Rng;
// use reqwest::StatusCode;
// use std::error::Error;
use std::sync::mpsc;
use std::time::Duration;
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
    Finishing,
    InProgress,
    Starting,
    OnQueue,
}

pub type JobId = u16;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Job {
    job_id: JobId,
    device_id: String,
    status: JobStatus,
    url: String,
    image: BinaryData,
}

#[allow(dead_code)]
pub struct JobScheduler {
    jobs: HashMap<JobId, Job>,
    on_queue: VecDeque<JobId>,
    running: Vec<JobId>,
    starting_job: Option<JobId>,
    finishing_job: Option<JobId>,
    last_running_job_index: u8, // TODO: Change this type
    messenger: Messenger,
    notification: mpsc::Receiver<Telemetry>,
}

impl JobScheduler {
    pub fn new(messenger: Messenger, notification: mpsc::Receiver<Telemetry>) -> Self {
        Self {
            jobs: HashMap::new(),
            on_queue: VecDeque::new(),
            running: Vec::new(),
            starting_job: None,
            finishing_job: None,
            last_running_job_index: 0,
            messenger,
            notification,
        }
    }

    pub fn run(&mut self) {
        // Just to simulate or testing purposes
        self.add_job(
            "device1".to_string(),
            "http://localhost:7777/bin/te1.txt".to_string(),
        );

        loop {
            // TODO: Here receive new job
            if let Ok(notif) = self.notification.recv_timeout(Duration::from_millis(100)) {
                self.handle_notification(notif);
            }

            // Start job from on_queue job list if running list not in max number
            if self.running.len() < MAX_RUNNING_JOB {
                if let Err(msg) = self.start_job_onqueue() {
                    println!("{msg}");
                    // TODO: do somekind of timeout for checking this if statement
                }
            }

            // Give cpu/thread some breath when no job in running status
            let Some(next_job_id) = self.get_next_job() else {
                // println!("No running job exists");
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
        let Some(job_id) = self.on_queue.pop_front() else {
            return Err(CustomError::StartJob(String::from(
                "No job that still OnQueue",
            )));
        };

        // Try to get job data (as mutable reference) to be modified later
        let Some(job) = self.jobs.get_mut(&job_id) else {
            return Err(CustomError::StartJob(format!(
                "No job in hashmap with id {}",
                job_id
            )));
        };

        // Send fota request command to target device
        let tosend =
            telemetry::build_command(job_id, &job.device_id, CommandType::OtaRequest).unwrap(); // TODO: handle error
        let _ = self.messenger.send(tosend);

        // Set the job as starting, also change the status on the real data
        self.starting_job = Some(job_id);
        job.status = JobStatus::Starting;

        Ok(())
    }

    fn start_job(&mut self, job_id: JobId) {
        // Try to get job data (as mutable reference) to be modified later
        let Some(job) = self.jobs.get_mut(&job_id) else {
            return; // TODO: Better error
        };

        // Attempt to download the binary from url provided
        match download_binary(&job.url) {
            Ok(data) => {
                // Now the binary already on the heap (BinaryData) and ready to chunked
                job.image = data;
                // Add the job to running index list
                self.running.push(job_id.clone());
                // Reset starting job to empty
                self.starting_job = None;
                // Change the actual job data status to in progres
                job.status = JobStatus::InProgress;
                // Ok(())
            }
            Err(msg) => {
                println!("job_id: {job_id} => {msg}");
                self.failed_job(job_id, "download firmware binary failed");
            }
        }
    }

    fn process_job(&mut self, job_id: JobId) {
        let Some(job) = self.jobs.get_mut(&job_id) else {
            // TODO: Return custom error
            // return Err(CustomError::StartJob(format!(
            //     "No job in hashmap with id {}",
            //     job_id
            // )));
            return;
        };

        match job.image.next() {
            Some(chunk) => {
                // Send fota request command to target device
                println!("data of {} => {:?}", job_id, chunk);
                let tosend = telemetry::build_packet(&job.device_id, 123, chunk);
                let _ = self.messenger.send(tosend); // TODO: Handle error
            }
            None => {
                if let Some(_) = self.finishing_job {
                    println!("Currently there's still job in finishing status {job_id}");
                    return;
                };

                // Finishing the job
                println!("Finishing job {job_id}");
                let tosend = telemetry::build_command(job_id, &job.device_id, CommandType::OtaDone);
                let _ = self.messenger.send(tosend.unwrap());
                // remove job from running list and change job status on hashmap
                job.status = JobStatus::Finishing;
                self.running.remove(self.last_running_job_index.into());
                // Add the job to finishing job candidate
                self.finishing_job = Some(job_id);
            }
        }
    }

    fn get_next_job(&mut self) -> Option<JobId> {
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

    fn failed_job(&mut self, job_id: JobId, reason: &str) {
        let Some(job) = self.jobs.get_mut(&job_id) else {
            return; // TODO: better error
        };
        job.status = JobStatus::Failed;
        self.starting_job = None;
        println!(
            "Job {} for device_id {} failed ({})",
            job_id, job.device_id, reason
        );
    }

    fn handle_notification(&mut self, notif: Telemetry) {
        let parsed = telemetry::parse(notif).unwrap(); // TODO: handle error

        if let Some(starting_job) = self.starting_job {
            if parsed.0 == starting_job {
                match parsed.1 {
                    CommandType::OtaRequestAck => self.start_job(starting_job),
                    CommandType::OtaRequestNack => self.failed_job(starting_job, "request denied"),
                    _ => return, // What happen here?,
                }
                return;
            }
        }

        if let Some(finishing_job) = self.finishing_job {
            if parsed.0 == finishing_job {
                let Some(job) = self.jobs.get_mut(&finishing_job) else {
                    return; // TODO: Better error
                };

                match parsed.1 {
                    CommandType::OtaDoneSuccess => job.status = JobStatus::Success,
                    CommandType::OtaDoneFailed => job.status = JobStatus::Failed,
                    _ => return, // What happen here,
                }

                // Whatever the result, consider it finish
                self.finishing_job = None;
                println!("Job {} is {:?}", job.job_id, job.status);
            }
        }
    }

    fn generate_job_id() -> JobId {
        let mut rng = rand::thread_rng();
        rng.gen_range(1000..=9999)
    }
}
