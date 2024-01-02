use rusb::{devices, DeviceDescriptor, DeviceHandle, GlobalContext};

use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(3);

fn _write_to_control(handle: DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(_n) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

fn _byte_vec_to_string(byte_vec: &mut Vec<u8>) -> Result<String, std::str::Utf8Error> {
    let mut string_vec: Vec<u8> = Vec::new();

    for &i in byte_vec.iter() {
        if i == 0 {
            break;
        }
        string_vec.push(i);
    }

    return match std::str::from_utf8(&string_vec) {
        Ok(string) => Ok(string.to_string()),
        Err(e) => return Err(e),
    };
}

fn _saftey_read(handle: DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let output: &mut Vec<u8> = &mut [0u8; 4096].to_vec();
    match handle.read_bulk(0x82, output, TIMEOUT) {
        Ok(_n) => (),
        Err(e) => match e {
            // We don't care if this timeouts
            rusb::Error::Timeout => (),
            _ => return Err(e),
        },
    };
    return Ok(());
}

fn _get_handle_from_vendor_product_id(
    vendor_id: u16,
    product_id: u16,
) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let devices = match devices() {
        Ok(list) => list,
        Err(e) => return Err(e),
    };

    for device in devices.iter() {
        let device_desc: DeviceDescriptor = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(e) => return Err(e),
        };

        if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
            return match device.open() {
                Ok(handle) => Ok(handle),
                Err(e) => return Err(e),
            };
        }
    }
    return Err(rusb::Error::NotFound);
}

pub fn read_from_bulk(handle: DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let raw_output: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    let response: String = match handle.read_bulk(0x82, raw_output, TIMEOUT) {
        Ok(_n) => match _byte_vec_to_string(raw_output) {
            Ok(r) => r,
            Err(e) => return Err(rusb::Error::Io),
        },
        Err(e) => return Err(e),
    };
    return Ok(response);
}

pub fn write_to_bulk(handle: DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written: usize = match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => n,
        Err(e) => return Err(e),
    };

    if bytes_written != command.len() {
        return Err(rusb::Error::Io);
    }
    return Ok(());
}

pub fn open(vendor_id: u16, product_id: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let mut handle: DeviceHandle<GlobalContext> =
        match _get_handle_from_vendor_product_id(vendor_id, product_id) {
            Ok(handle) => handle,
            Err(e) => return Err(e),
        };

    match handle.claim_interface(0) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    match _write_to_control(handle, 2) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    match _saftey_read(handle) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    return Ok(handle);
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match _write_to_control(*handle, 4) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    match handle.release_interface(0) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn send_command_get_response(
    handle: DeviceHandle<GlobalContext>,
    command: &[u8],
) -> rusb::Result<String> {
    match _saftey_read(handle) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    match write_to_bulk(handle, command) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    return match read_from_bulk(handle) {
        Ok(r) => Ok(r),
        Err(e) => return Err(e),
    };
}
