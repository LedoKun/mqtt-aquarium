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
            sensor.read(&mut buffer).unwrap();

            let firmware = String::from_utf8(buffer.to_vec()).unwrap();

            debug!("Firmware = {}", firmware);
    
            if firmware != FIRMWARE {
                return Err(format!("Unsupported firmware: {}", firmware));
            }
            
            // First read always delivers garbadge
            sensor.write(&[0x01, 0x80, 0x33, 0x01, 0, 0, 0, 0]).unwrap();
            sensor.read(&mut buffer).unwrap();

            // for buffer = internal temp and buffer2 = external temp (probe)
            let mut buffer2 = [0u8; 8];
            sensor.write(&[0x01, 0x80, 0x33, 0x01, 0, 0, 0, 0]).unwrap();
            sensor.read(&mut buffer).unwrap();
            sensor.read(&mut buffer2).unwrap();

            debug!("Buffer = {:x?}", buffer);
            debug!("Buffer2 = {:x?}", buffer2);

            let inside_temp = convert_temp(buffer[2], buffer[3]);
            let outside_temp = convert_temp(buffer2[2], buffer2[3]);

            debug!("Outside temp = {}", inside_temp);
            debug!("Inside temp = {}", outside_temp);

            return Ok((inside_temp, outside_temp));
        }
    }

    return Err("Unable to read temperature!".to_string());
}
