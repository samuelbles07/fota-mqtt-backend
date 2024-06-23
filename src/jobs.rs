use crate::custom_error::CustomError;
use crate::file_handler::{download_binary, BinaryData};
use crate::messenger::Messenger;
use crate::telemetry::{self, CommandType, Telemetry};
use core::time;
use rand::Rng;
use serde::Deserialize;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{
    collections::{HashMap, VecDeque},
    thread,
};

// TODO: value from configuration
const MAX_RUNNING_JOB: usize = 3;
const JOB_PROCESSED_INTERVAL: Duration = Duration::from_millis(200); // Expected process interval for job

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

#[derive(Debug)]
pub struct Job {
    job_id: JobId,
    device_id: String,
    status: JobStatus,
    url: String,
    image: BinaryData,
    last_time_processed: Instant,
}

#[derive(Debug, Deserialize)]
pub struct NewJob {
    device_id: String,
    url: String,
}

pub struct JobScheduler {
    jobs: HashMap<JobId, Job>,
    on_queue: VecDeque<JobId>,
    running: Vec<JobId>,
    starting_job: Option<JobId>,
    finishing_job: Option<JobId>,
    last_running_job_index: u8, // TODO: Change this type
    messenger: Messenger,
    ch_notification: mpsc::Receiver<Telemetry>, // TODO: Change name to ch_notification
    ch_new_job: mpsc::Receiver<NewJob>,
}

impl JobScheduler {
    pub fn new(
        messenger: Messenger,
        rx_notification: mpsc::Receiver<Telemetry>,
        rx_new_job: mpsc::Receiver<NewJob>,
    ) -> Self {
        Self {
            jobs: HashMap::new(),
            on_queue: VecDeque::new(),
            running: Vec::new(),
            starting_job: None,
            finishing_job: None,
            last_running_job_index: 0,
            messenger,
            ch_notification: rx_notification,
            ch_new_job: rx_new_job,
        }
    }

    pub fn run(self) {
        info!("Run jobs thread");
        thread::spawn(move || self._run());
    }

    fn _run(mut self) {
        loop {
            if let Ok(new_job) = self.ch_new_job.recv_timeout(Duration::from_millis(10)) {
                info!("Receive new job: {new_job:?}");
                self.add_job(new_job);
            }

            if let Ok(notif) = self.ch_notification.recv_timeout(Duration::from_millis(10)) {
                self.handle_notification(notif);
            }

            // Start job from on_queue job list if running list not in max number
            if self.running.len() < MAX_RUNNING_JOB {
                if let Err(msg) = self.start_job_onqueue() {
                    error!("Starting job err ({msg})");
                    // TODO: do somekind of interval for checking this if statement
                }
            }

            // Give cpu/thread some breath when no job in running status
            let Some(next_job_id) = self.get_next_job() else {
                trace!("No running job exists");
                thread::sleep(time::Duration::from_secs(1));
                continue;
            };

            debug!("Next Job: {next_job_id}");
            self.process_job(next_job_id);

            thread::sleep(Duration::from_millis(10)); // To give breath to the cpu
        }
    }

    fn add_job(&mut self, new_job: NewJob) {
        // Add new job to the on_queue list
        let job_id = Self::generate_job_id();
        self.jobs.insert(
            job_id,
            Job {
                job_id,
                device_id: new_job.device_id,
                status: JobStatus::OnQueue,
                url: new_job.url,
                image: BinaryData::default(),
                last_time_processed: Instant::now(),
            },
        );

        self.on_queue.push_back(job_id);
        trace!("New job added {:#?}", self.jobs.get(&job_id));
    }

    fn start_job_onqueue(&mut self) -> Result<(), CustomError> {
        // Get job that still on queue.
        // Only get reference since, needs to process it first before removing it from the queue
        let Some(job_id) = self.on_queue.pop_front() else {
            trace!("No job in the queue");
            return Ok(());
        };

        // Try to get job data (as mutable reference) to be modified later
        let Some(job) = self.jobs.get_mut(&job_id) else {
            return Err(CustomError::StartJob(format!(
                "No job in hashmap with id {}",
                job_id
            )));
        };

        // Attempt to download the binary from url provided
        debug!("Attempt download binary image of job {job_id}");
        match download_binary(&job.url) {
            Ok(data) => {
                // Now the binary already on the heap (BinaryData) and ready to chunked
                job.image = data;
                // Send fota request command to target device
                // TODO: Send hash too
                let tosend = telemetry::build_command(
                    job_id,
                    &job.device_id,
                    CommandType::OtaRequest,
                    &job.image.hash,
                )
                .unwrap(); // TODO: handle error
                let _ = self.messenger.send(tosend); // TODO: Handle error
                debug!("fota request is sent to {}", job.device_id);

                // Set the job as starting, also change the status on the real data
                self.starting_job = Some(job_id);
                job.status = JobStatus::Starting;
                info!("Job {job_id} is starting");

                Ok(())
            }
            Err(msg) => {
                warn!("Download file failed for {job_id} ({msg})");
                self.failed_job(job_id, "download firmware binary failed");
                Err(CustomError::StartJob(String::from("Failed download")))
            }
        }
    }

    fn start_job(&mut self, job_id: JobId) {
        // Try to get job data (as mutable reference) to be modified later
        let Some(job) = self.jobs.get_mut(&job_id) else {
            return; // TODO: Better error
        };

        // Add the job to running index list
        self.running.push(job_id.clone());
        // Reset starting job to empty
        self.starting_job = None;
        // Change the actual job data status to in progres
        job.status = JobStatus::InProgress;
        info!("Job {job_id} now in progress");

        // Set the last time job is processed
        job.last_time_processed = Instant::now();
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

        // Delay before process the next job when interval is not met yet
        thread::sleep(JobScheduler::get_job_interval_delay(
            job_id,
            job.last_time_processed,
        ));
        job.last_time_processed = Instant::now(); // Set the new clock

        match job.image.next() {
            Some(chunk) => {
                // Send fota request command to target device
                debug!("data of {} => {:?}", job_id, job.image.last_bytes_index);
                let tosend =
                    telemetry::build_packet(&job.device_id, job.image.current_chunk_id, chunk);
                let _ = self.messenger.send(tosend); // TODO: Handle error
            }
            None => {
                if let Some(_) = self.finishing_job {
                    info!("Currently there's still job in finishing status {job_id}");
                    return;
                };

                // Finishing the job
                info!("Finishing job {job_id}");
                let tosend = telemetry::build_command(
                    job_id,
                    &job.device_id,
                    CommandType::OtaDone,
                    &Vec::new(), // Just send empty vector, done command don't need image hash in the payload
                );
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
        warn!(
            "Job {} for device_id {} failed ({})",
            job_id, job.device_id, reason
        );
    }

    fn handle_notification(&mut self, notif: Telemetry) {
        let parsed = telemetry::parse(notif).unwrap(); // TODO: handle error

        // Check if notification for job that currently starting
        if let Some(starting_job) = self.starting_job {
            if parsed.0 == starting_job {
                debug!("Notification for currently starting job");
                match parsed.1 {
                    CommandType::OtaRequestAck => self.start_job(starting_job),
                    CommandType::OtaRequestNack => self.failed_job(starting_job, "request denied"),
                    _ => return, // What happen here?,
                }
                return;
            }
        }

        // Check if notification for job that currently finishing
        if let Some(finishing_job) = self.finishing_job {
            if parsed.0 == finishing_job {
                debug!("Notification for currently finishing job");
                let Some(job) = self.jobs.get_mut(&finishing_job) else {
                    return; // TODO: Better error
                };

                match parsed.1 {
                    CommandType::OtaDoneSuccess => {
                        job.status = JobStatus::Success;
                        info!("Job {} is SUCCESS", job.job_id);
                    }
                    CommandType::OtaDoneFailed => {
                        job.status = JobStatus::Failed;
                        warn!("Job {} is FAILED", job.job_id);
                    }
                    _ => return, // TODO: What happen here,
                }

                // Whatever the result, consider it finish
                self.finishing_job = None;
            }
        }
    }

    fn get_job_interval_delay(job_id: JobId, last_interval: Instant) -> Duration {
        let elapsed = last_interval.elapsed();

        if JOB_PROCESSED_INTERVAL <= elapsed {
            warn!(
                "Job id {} interval missed by {:?}",
                job_id,
                elapsed - JOB_PROCESSED_INTERVAL
            );
            return Duration::new(0, 0);
        }

        let dur = JOB_PROCESSED_INTERVAL - elapsed;
        debug!("Job id {} early by {:?}", job_id, dur);
        dur
    }

    fn generate_job_id() -> JobId {
        let mut rng = rand::thread_rng();
        rng.gen_range(1000..=9999)
    }
}
