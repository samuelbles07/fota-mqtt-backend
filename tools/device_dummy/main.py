import random
import argparse
from typing import Any, Optional
from queue import Queue
from enum import Enum

import paho.mqtt.client as mqtt
import cbor2 as cbor


class CommandType(Enum):
    OTA_REQUEST = 0x01
    OTA_REQUEST_ACK = 0x02
    OTA_REQUEST_NACK = 0x03
    OTA_DONE = 0x04
    OTA_DONE_SUCCESS = 0x05
    OTA_DONE_FAILED = 0x06


def on_connect(client: mqtt.Client, userdata, flags, reason_code, properties):
    print(f"Connected with result code {reason_code}")
    # Subscribe everytime client is connected
    client.subscribe("/fota/cmd/+")
    client.subscribe("/fota/data/+/+")


# The callback for when a PUBLISH message is received from the server.
def on_message(client: mqtt.Client, userdata: Queue, msg: mqtt.MQTTMessage):
    if is_request_data(msg.topic):
        print(f"Chunk received: {msg.topic}")
        return

    # Only add command request to the queue
    userdata.put((msg.topic, msg.payload)) # Tuple (topic, payload)

def handle_command(topic: str, payload: bytes) -> Optional[tuple[str, bytes]]: 
    # Decode payload
    data = cbor.loads(payload)

    print("\n-----------------------")
    print(f"JobId: {data[0]}")

    response_command = None
    match data[1]:
        case CommandType.OTA_REQUEST.value:
            print("Command is {}".format(CommandType.OTA_REQUEST))
            print("Pick response: ")
            print("1. OTA_REQUEST_ACK")
            print("2. OTA_REQUEST_NACK")
            pick = int(input())
            if pick == 1:
                response_command = CommandType.OTA_REQUEST_ACK.value
            elif pick == 2:
                response_command = CommandType.OTA_REQUEST_ACK.value
        case CommandType.OTA_DONE.value:
            print("Command is {}".format(CommandType.OTA_DONE))
            print("Pick response: ")
            print("1. OTA_DONE_SUCCESS")
            print("2. OTA_DONE_FAILED")
            pick = int(input())
            if pick == 1:
                response_command = CommandType.OTA_DONE_SUCCESS.value
            elif pick == 2:
                response_command = CommandType.OTA_DONE_FAILED.value
        case _:
            pass

    # Make sure not empty
    if response_command is None:
        print("Unknown command or bad pick, ignore")
        return None
    
    print("-----------------------\n")
    
    # Encode payload
    resp_payload = cbor.dumps( [data[0], response_command] )

    # Remake topic
    resp_topic = topic.replace("cmd", "cmd_resp") 

    return (resp_topic, resp_payload)


def is_request_data(topic: str) -> bool:
    ''' Check if topic is for binary chunk
    '''
    split = topic.split("/")
    # print(split)
    if split[2] == "data":
        return True

    return False


def connect_mqtt(mqtt_host: str, mqtt_port: int, downlink_queue: Queue) -> mqtt.Client:
    client_id = f'python-mqtt-{random.randint(0, 1000)}'
    client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id) 
    client.on_connect = on_connect
    client.on_message = on_message
    client.user_data_set(downlink_queue)
    # client.username_pw_set("user", "pwd")
    # client.tls_set(CA_CERT_PATH, tls_version=ssl.PROTOCOL_TLSv1_2)#, cert_reqs=ssl.CERT_OPTIONAL)
    # client.tls_insecure_set(True)
    client.connect(mqtt_host, mqtt_port)
    return client


if __name__ == "__main__":
    # Parse arguments
    parser = argparse.ArgumentParser(description="Set mqtt host")
    parser.add_argument("--host", action="store", help="MQTT Host", required=True)
    parser.add_argument("--port", action="store", help="MQTT Port", type=int, required=True)
    args = parser.parse_args()

    # Create queue for mqtt thread send downlink main thread
    downlink_queue:Queue = Queue() 

    # Connect MQTT connection
    mqtt_client = connect_mqtt(args.host, args.port, downlink_queue)

    # Start MQTT eventloop in new thread
    mqtt_client.loop_start()
    print("Start")

    while True:
        try:
            topic, payload = downlink_queue.get()
            if (result := handle_command(topic, payload)) is None:
                # If request command unknown, will not respond downlink
                continue
            # Send response
            topic, payload = result 
            mqtt_client.publish(topic, payload, qos=1)
        except KeyboardInterrupt:
            break 

    mqtt_client.loop_stop()
    print("exit!")
