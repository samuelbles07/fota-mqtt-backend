use jobs::JobScheduler;

mod custom_error;
mod file_handler;
mod jobs;

fn main() {
    let mut job_scheduler = JobScheduler::new();
    job_scheduler.add_job("Musang".to_string(), "Heyo".to_string());
    // let mut job1 =
    // let url: String = "http://localhost:7777/bin/tes.txt".to_string();
    // let result = file_handler::download_binary(&url);
    // match result {
    //     Ok(data) => mydata.data = Bytes::from(data),
    //     Err(err) => {
    //         println!("Error: {err}");
    //         std::process::exit(1);
    //     }
    // };
    //
    // for val in mydata {
    //     println!("{val:?}");
    // }

    // println!("{:?}", result);
}
