MQTT Aquarium
===========

An [Rust](https://www.rust-lang.org/) implementation of an MQTT client, using [rumqttc](https://docs.rs/rumqttc/latest/rumqttc/), to publish readings from TEMPer2 sensors.

An MQTT client implemented in [Rust](https://www.rust-lang.org/) using [rumqttc](https://docs.rs/rumqttc/latest/rumqttc/) and [HidApi](https://crates.io/crates/hidapi) to publish temperature readings from TEMPer2 usb devices and support [Home Assistant](https://www.home-assistant.io/) MQTT devices autodiscovery. The program only supports **1a86xe025** devices with *TEMPer2_* firmware.

Most of the USB sequence is copied from [rust-temper](https://github.com/flxo/rust-temper)!

This project is just a hack, done in order to learn Rust (and to monitor the temperature of my aquariums in [Home Assistant](https://www.home-assistant.io/)!).

# Build and Run

```sh
$ cargo build
[...]
$ sudo cargo run
     Running `target/debug/mqtt-aquarium`
[...]
```

# Build and Run Docker container

```sh
$ docker build -t mqtt-aquarium .
[...]
$ docker run --privileged -v /dev:/dev -e RUST_LOG=info -e MQTT_SERVER=mqtt -e MQTT_PORT=1883 mqtt-aquarium
```

# License 

MIT License (MIT). Copyright (c) 2022 Rom Luengwattanapong
