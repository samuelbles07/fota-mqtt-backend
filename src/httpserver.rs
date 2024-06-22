use std::sync::mpsc;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug)]
pub struct HTTPServer {
    listener: TcpListener,
    ch_new_job: mpsc::Sender<String>,
}

impl HTTPServer {
    pub fn new(host: &str, port: u16, tx_new_job: mpsc::Sender<String>) -> Self {
        let listener = TcpListener::bind(format!("{host}:{port}")).unwrap();
        info!("Bind {host} on port {port}");
        Self {
            listener,
            ch_new_job: tx_new_job,
        }
    }

    pub fn run(&mut self) {
        info!("Running http server");
        for stream in self.listener.incoming() {
            // TODO: Better error handling
            let stream = stream.unwrap();
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let mut buf_reader = BufReader::new(&mut stream);
        let mut request_line = String::new();

        // Request line
        buf_reader.read_line(&mut request_line).unwrap();
        debug!("request_line: {request_line}");

        // println!("{http_request:#?}");
        if request_line == "POST /job HTTP/1.1\r\n" {
            self.handle_post_job(buf_reader);
            let response = "HTTP/1.1 201 Created\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        } else {
            // some other request
            let status_line = "HTTP/1.1 404 NOT FOUND";
            // let contents = fs::read_to_string("404.html").unwrap();
            let contents = String::from("404 NOT FOUND");
            let length = contents.len();

            let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

            stream.write_all(response.as_bytes()).unwrap();
        }
    }

    fn handle_post_job(&self, mut reader: BufReader<&mut TcpStream>) {
        // Read header and get the content length
        let mut list_header: Vec<String> = Vec::new();
        let mut content_len: u32 = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }

            if line.len() > 15 {
                if &line[0..14] == "Content-Length" {
                    trace!("Content-Length found");
                    content_len = self.get_content_length(&line[..line.len() - 2]);
                }
            }

            list_header.push(line[..line.len() - 2].into());
        }

        debug!("header: {list_header:#?}");
        debug!("cl: {content_len}");

        // Read the content
        let mut buf = vec![0; content_len as usize];
        reader.read_exact(&mut buf).unwrap();
        let content = String::from_utf8(buf).expect("Not a valid utf8");
        debug!("content: {content}");
        self.ch_new_job.send(content).unwrap(); // TODO: Handle error
    }

    fn get_content_length(&self, str: &str) -> u32 {
        let splitted: Vec<&str> = str.split(":").collect();
        let len_str = splitted[1].trim();
        len_str.parse::<u32>().unwrap()
    }
}
