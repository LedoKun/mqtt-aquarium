use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Serialize;
use std::env;
use std::time::Duration;
use tokio::{task, time};

extern crate pretty_env_logger;

#[macro_use]
extern crate log;

mod temper2;

#[derive(Serialize)]
struct MQTTPayload {
    temperature: f32,
}

#[derive(Serialize)]
struct MQTTTempPayload {
    outside_temperature: f32,
    inside_temperature: f32,
}

#[derive(Clone, Serialize, Debug)]
struct MQTTADPayload {
    state_topic: String,
    state_class: String,
    device_class: String,
    unit_of_measurement: String,
    name: String,
    icon: String,
    value_template: String,
    unique_id: String,
}

fn get_unique_id() -> String {
    let hostname = std::fs::read_to_string("/etc/hostname")
        .unwrap()
        .trim()
        .to_string();

    debug!("Got hostname = {hostname}");

    hostname
}

#[tokio::main]
async fn main() {
    const MQTT_CLIENT_NAME: &str = "mqtt-aquarium";
    const CONFIG_TOPIC: &str = "homeassistant/sensor/aquariumTemp/config";
    const STATE_TOPIC: &str = "homeassistant/sensor/aquariumTemp/state";

    // Force log level to info, if none set!
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init_timed();

    // Get MQTT broker server & port
    let mqtt_server = env::var("MQTT_SERVER").unwrap_or("localhost".to_string());
    let mqtt_port = env::var("MQTT_PORT").unwrap_or("1883".to_string());

    info!("Stating MQTT Client");
    info!("Set log level via env \"RUST_LOG\"");
    info!("MQTT server = {}, port = {}", mqtt_server, mqtt_port);

    let mut mqttoptions = MqttOptions::new(
        MQTT_CLIENT_NAME,
        mqtt_server,
        mqtt_port.parse::<u16>().unwrap(),
    );
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    debug!("Client = {:?}", client);

    // Prepare auto discovery payloads
    let room_temp = MQTTADPayload {
        state_topic: STATE_TOPIC.to_string(),
        state_class: "measurement".to_string(),
        device_class: "temperature".to_string(),
        unit_of_measurement: "°C".to_string(),
        name: "Room Temperature".to_string(),
        icon: "mdi:thermometer".to_string(),
        value_template: "{{ value_json.outside_temperature }}".to_string(),
        unique_id: format!("aquarium_{}", get_unique_id()),
    };

    let mut aquarium_temp = room_temp.clone();
    aquarium_temp.name = "Aquarium Temperature".to_string();
    aquarium_temp.value_template = "{{ value_json.inside_temperature }}".to_string();

    debug!("room_temp_AD = {:#?}", room_temp);
    debug!("aquarium_temp_AD = {:#?}", aquarium_temp);

    task::spawn(async move {
        loop {
            let sensor_data = temper2::read_temp();

            match sensor_data {
                Err(e) => {
                    error!("Sensor error = {}", e);
                }
                Ok((out_temp, in_temp)) => {
                    let temperature_payload = MQTTTempPayload {
                        outside_temperature: out_temp,
                        inside_temperature: in_temp,
                    };

                    //publish autodiscovery payload
                    let publish_room_ad = client
                        .publish(
                            CONFIG_TOPIC,
                            QoS::AtLeastOnce,
                            false,
                            serde_json::to_string(&room_temp).unwrap(),
                        )
                        .await;

                    let publish_aquarium_ad = client
                        .publish(
                            CONFIG_TOPIC,
                            QoS::AtLeastOnce,
                            false,
                            serde_json::to_string(&aquarium_temp).unwrap(),
                        )
                        .await;

                    //publish common temperature payload
                    let publish_result = client
                        .publish(
                            STATE_TOPIC,
                            QoS::AtLeastOnce,
                            false,
                            serde_json::to_string(&temperature_payload).unwrap(),
                        )
                        .await;

                    match (publish_room_ad, publish_aquarium_ad, publish_result) {
                        (Ok(_), Ok(_), Ok(_)) => {
                            info!(
                                "Published temperature outside = {}°C, inside = {}°C",
                                temperature_payload.outside_temperature,
                                temperature_payload.inside_temperature
                            );
                        }
                        _ => error!("Error published data to MQTT broker!"),
                    }
                }
            }

            time::sleep(Duration::from_secs(10)).await;
        }
    });

    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(rumqttc::Event::Incoming(rumqttc::Incoming::ConnAck(msg))) => {
                info!("Connected to the broker!");
                debug!("Connected msg = {msg:?}");
            }
            Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect)) => {
                warn!("Disconnected, retry happening...");
            }
            Ok(msg) => {
                debug!("Event = {msg:?}");
            }
            Err(e) => {
                error!("Error = {}", e);
                error!("Terminating...");
                break;
            }
        }
    }
}
