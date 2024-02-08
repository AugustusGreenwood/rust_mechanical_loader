use rusb::{devices, DeviceDescriptor, DeviceHandle, GlobalContext};

use std::{string::FromUtf8Error, time::Duration};

const TIMEOUT: Duration = Duration::from_secs(3);

// When a bulk_read returns, it will be a byte vector which will look like:
// [#, #, #, 0, ...] where ... is garbage after the null byte we need to ignore.
// This removes all the garbage after the null byte and then converts it to a String.
fn byte_vec_to_string(byte_vec: &Vec<u8>) -> Result<String, FromUtf8Error> {
    let mut string_vec: Vec<u8> = Vec::new();

    for &i in byte_vec.iter() {
        if i == 0 {
            break;
        }
        string_vec.push(i);
    }
    return String::from_utf8(string_vec);
}

pub fn write_to_control(handle: &DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    let _ = handle.write_control(64, 2, value, 0, &[], TIMEOUT)?;
    return Ok(());
}

pub fn saftey_read(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    // This will almost always error (usually timout) and doesn't effect
    // communication so all errors are ignored. We also don't care how many
    // bytes were written. just full of apathy
    let _ = handle.read_bulk(0x82, &mut [0u8; 64], TIMEOUT);
    return Ok(());
}

pub fn get_handle_from_vendor_product_id(
    vendor_id: u16,
    product_id: u16,
) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let devices = devices()?;

    for device in devices.iter() {
        let device_desc: DeviceDescriptor = device.device_descriptor()?;

        if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
            return device.open();
        }
    }
    return Err(rusb::Error::NotFound);
}

pub fn read_from_bulk(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let raw_output = &mut [0u8; 64].to_vec();
    handle.read_bulk(0x82, raw_output, TIMEOUT)?;
    return Ok(byte_vec_to_string(raw_output).unwrap());
}

pub fn write_to_bulk(handle: &DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written = handle.write_bulk(0x02, command, TIMEOUT)?;

    if bytes_written != command.len() {
        return Err(rusb::Error::Io);
    }
    return Ok(());
}
