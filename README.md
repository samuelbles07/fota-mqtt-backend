# Rocky

<p align="center">
  <img src="docs/rocky.png" alt="rocky-patrick"/>
</p>

## Description

A simple backend service for managing Over-The-Air (OTA) firmware updates via MQTT written in rust. Designed as a job scheduler, it handles multiple firmware update requests simultaneously, with each job maintaining its own status. 

> This is my learning project while studying Rust. While error handling still bad, it already demonstrates how OTA firmware updates happen through MQTT. It's sufficient for testing and learning purposes. In the future, I plan to improve it for use in a production environment. 

## How it works?

**Here add general flow diagram**

It has 3 threads which are the main entity in this project `httpserver`, `jobs` and `messenger`. 

- `httpserver` → where user able to create new job through http request
- `jobs` → manage job schedule in 1 thread. It will process through a list of _in-progress_ job consecutively, set job that still _on-queue_ to _in-progress_ list and set job that is finished to _success_ or _failed_ status. 
- `messenger` → handle mqtt connection pool, publish messages and forward notification to jobs 

### Channel

There are 2 channel for communication between threads. **Notification** channel to send incoming message from `messenger` to `jobs` thread and **new job** channel to send new job from `httpserver` to `jobs` thread.

**Here add simple diagram about channel**

### MQTT Topic and Payload

#### Command

There are 2 command topic

- `/fota/cmd/{device_id}` → where service send fota command request to device
- `/fota/cmd_resp/{device_id}` → where device respond fota command request to service 

**Payload**



### Device Dummy

## How to run

blablabla
