# Rocky

<img align="right" src="docs/rocky.png" alt="Rocky logo">

A simple backend service for managing Over-The-Air (OTA) firmware updates via MQTT written in rust. Designed as a job scheduler, it handles multiple firmware update requests simultaneously, with each job maintaining its own status. 

> This is my learning project while studying Rust. While error handling still bad, it already demonstrates how OTA firmware updates happen through MQTT. It's sufficient for testing and learning purposes. In the future, I plan to improve it for use in a production environment. 

## How it works?

```mermaid
sequenceDiagram

actor User
participant http as HTTP Server
box Aqua Channel
participant newjob as NewJob
end
participant jobs as Jobs 
box Aqua Channel
participant notif as Notification
end
participant msg as Messenger

par Request
User->>+http: request new job
http->>newjob: send(NewJob)
http-->>-User: response(success/failed)

and JobScheduler
loop Job Interval
newjob->>jobs: recv(NewJob)
jobs->>jobs: add_job(queue)
notif->>jobs: recv(Telemetry)
jobs->>jobs: handle_notification
jobs->>jobs: process jobs 
jobs->>msg: publish_message(Telemetry)
end

and MQTT Connection Pool
msg->>notif: send(Telemetry) 
end
```

It has 3 threads which are the main entity in this project `httpserver`, `jobs` and `messenger`. 

- `httpserver` → where user able to create new job through http request
- `jobs` → manage job schedule in 1 thread. It will process through a list of _in-progress_ job consecutively, set job that still _on-queue_ to _in-progress_ list and set job that is finished to _success_ or _failed_ status. 
- `messenger` → handle mqtt connection pool, publish messages and forward notification to jobs 

### Processing Job 

Each job is processed as shown in the diagram below:

```mermaid 
sequenceDiagram

participant r as Rocky 
participant d as IoT Device 

r->>d: send_cmd(FOTA_REQUEST)
alt request denied 
d-->>r: send_cmd(FOTA_REQUEST_NACK)
else request accepted
d-->>r: send_cmd(FOTA_REQUEST_ACK)
r->>d: send_data(chunk)
Note right of d: repeat until last binary chunk
r->>d: send_cmd(FOTA_DONE)
d-->>r: send_cmd(FOTA_DONE_SUCCESS / FOTA_DONE_FAILED)
end
```

### Channel

There are 2 channel for communication between threads. **Notification** channel to send incoming message from `messenger` to `jobs` thread and **NewJob** channel to send new job from `httpserver` to `jobs` thread.

### MQTT Topic and Payload

#### Command

There are 2 command topic

- `/fota/cmd/{device_id}` → where service send fota command request to device
- `/fota/cmd_resp/{device_id}` → where device respond fota command to service 

**Payload**

For command request, payload is encoded using cbor, with plain text as follow 

`[ {job_id<4 digit integer>}, {command_type<1byte>}, [{image_hash<32byte array>}] ]`

`image_hash` is device firmware binary hashed using sha256, so `image_hash` value is alwasy 32 bytes. Also, `image_hash` is only exist for command type `FOTA_REQUEST`.

**Command Type**

|Command Type|Value|Topic
|------------|-----|-----|
|FOTA_REQUEST|0x01 |`cmd`|
|FOTA_REQUEST_ACK|0x02 |`cmd_resp`|
|FOTA_REQUEST_NACK|0x03 |`cmd_resp`|
|FOTA_DONE|0x04 |`cmd`|
|FOTA_DONE_SUCCESS|0x05 |`cmd_resp`|
|FOTA_DONE_FAILED|0x06 |`cmd_resp`|


#### Data

Only 1 topic `/fota/data/{device_id}/{chunk_id}`. `chunk_id` is identifier for each chunk that is sent alongside the actual binary chunk in the payload. Binary chunk is not encoded, formatted or anything, just straight forward.

> `chunk_id` will be used in the future for resume or resend purposes

### Configuration

Set configuration value in `rocky.toml`

## How to run

Just directly run like `$ RUST_LOG=info cargo run`, change log level as needed. Or if don't have rust environment, just use docker.

**Build**

```sh
$ docker build -t rocky .
```

**Run**

```sh 
$ docker run -d --rm -p 7777:7777 -e RUST_LOG=info -v ${PWD}/rocky.toml:/rocky.toml --name rocky rocky
```

**Note**

> Use device_dummy tools on `tools/device_dummy` to simulate the iot device end

#### Request Sample

```sh 
$ curl -X POST http://localhost:7878/job \
    --header "Content-Type: application/json" \
    --data '{"device_id":"musang", "url":"http://domain.com:7777/bin/test3.txt"}' -v
```
