FROM rust:slim-bullseye AS builder
ENV DEBIAN_FRONTEND=noninteractive
COPY . /tmp/mqtt-aquarium/
RUN cd /tmp/mqtt-aquarium/ \
        && apt-get update \
        && apt-get install -y \
                build-essential \
                pkg-config \
                libusb-1.0-0-dev \
        && cargo build --release

FROM debian:bullseye-slim
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_LOG=info
ENV MQTT_SERVER=mqtt
ENV MQTT_PORT=1883
COPY --from=builder /tmp/mqtt-aquarium/target/release/mqtt-aquarium .
RUN apt-get update \
        && apt-get install -y --no-install-recommends \
                tini \
                libusb-1.0-0 \
        && apt-get clean \
        && chmod +x ./mqtt-aquarium
ENTRYPOINT ["/usr/bin/tini-static", "--"]
CMD [ "./mqtt-aquarium" ]