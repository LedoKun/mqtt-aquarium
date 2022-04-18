use hidapi::HidApi;

extern crate pretty_env_logger;

fn convert_temp(b2: u8, b3: u8) -> f32 {
    let b2 = b2 as u16;
    let b3 = b3 as u16;

    let t: i16 = (b2 as i16) << 8u32 | b3 as i16;
    let temperature = (t as f32) / 100.0;

    return temperature;
}

pub fn read_temp() -> Result<(f32, f32), String> {
    const FIRMWARE: &str = "TEMPer2_";

    const VID: u16 = 0x1a86;
    const PID: u16 = 0xe025;
    const INTERFACE_NUMBER: i32 = 1;

    const SENSOR_READ_TIMEOUT_MS: i32 = 1000;

    let api = HidApi::new().unwrap();

    for device in api.device_list() {
        debug!(
            "Device = {:#06x}:{:#06x} - interface_number = {:?}",
            device.vendor_id(),
            device.product_id(),
            device.interface_number()
        );

        if device.vendor_id() == VID
            && device.product_id() == PID
            && device.interface_number() == INTERFACE_NUMBER
        {
            let sensor = api.open_path(device.path()).unwrap();

            let mut buffer = [0u8; 8];
            sensor.write(&[0x01, 0x86, 0xff, 0x01, 0, 0, 0, 0]).unwrap();
            sensor
                .read_timeout(&mut buffer, SENSOR_READ_TIMEOUT_MS)
                .unwrap();

            let firmware = String::from_utf8(buffer.to_vec()).unwrap();

            debug!("Firmware = {}", firmware);
            if firmware != FIRMWARE {
                return Err(format!("Unsupported firmware: {}", firmware));
            }
            // First read always delivers garbadge
            sensor.write(&[0x01, 0x80, 0x33, 0x01, 0, 0, 0, 0]).unwrap();
            sensor
                .read_timeout(&mut buffer, SENSOR_READ_TIMEOUT_MS)
                .unwrap();

            // for buffer = internal temp and buffer2 = external temp (probe)
            let mut buffer2 = [0u8; 8];
            sensor.write(&[0x01, 0x80, 0x33, 0x01, 0, 0, 0, 0]).unwrap();
            sensor
                .read_timeout(&mut buffer, SENSOR_READ_TIMEOUT_MS)
                .unwrap();
            sensor
                .read_timeout(&mut buffer2, SENSOR_READ_TIMEOUT_MS)
                .unwrap();

            debug!("Buffer = {:x?}", buffer);
            debug!("Buffer2 = {:x?}", buffer2);

            if buffer[0] != 0x80 || (buffer[2] == 0x4e && buffer[3] == 0x20) {
                return Err("Unable to read inside temperature!".to_string());
            } else if buffer2[0] != 0x80 || (buffer2[2] == 0x4e && buffer2[3] == 0x20) {
                return Err("Unable to read outside (probe) temperature!".to_string());
            }

            let inside_temp = convert_temp(buffer[2], buffer[3]);
            let outside_temp = convert_temp(buffer2[2], buffer2[3]);

            debug!("Outside temp = {}", inside_temp);
            debug!("Inside temp = {}", outside_temp);

            return Ok((inside_temp, outside_temp));
        }
    }

    return Err("Something is wrong with the sensor!".to_string());
}
