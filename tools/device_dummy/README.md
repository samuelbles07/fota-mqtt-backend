## Device Simulation

Simulate device by accept any *client_id* from the topic and manually pick response for each command request.


```sh 
Start
Connected with result code Success

-----------------------
JobId 	: 9394
Image Hash : [199, 222, 72, 164, 24, 0, 25, 38, 125, 68, 183, 253, 213, 25, 191, 186, 5, 127, 61, 208, 216, 32, 11, 89, 53, 248, 138, 95, 53, 122, 63, 186]
Command is CommandType.OTA_REQUEST
Pick response:
1. OTA_REQUEST_ACK
2. OTA_REQUEST_NACK
1
-----------------------

Chunk received: /fota/data/device1/1
Chunk received: /fota/data/device1/2
Chunk received: /fota/data/device1/3
Chunk received: /fota/data/device1/4
Chunk received: /fota/data/device1/5
Chunk received: /fota/data/device1/6
Chunk received: /fota/data/device1/7
Chunk received: /fota/data/device1/8

-----------------------
JobId 	: 9394
Image Hash : []
Command is CommandType.OTA_DONE
Pick response:
1. OTA_DONE_SUCCESS
2. OTA_DONE_FAILED
2
-----------------------
```
