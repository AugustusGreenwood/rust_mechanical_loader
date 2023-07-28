use std::{
    time::Duration,
    io::Write,
};

use rusb::{
    DeviceHandle, devices, GlobalContext, DeviceDescriptor
};

// Timeout used for all operations. You could change it, but it's not really
// worth it. The timeout is never the limiting thing except when something 
// else is wrong, so I would just leave it at 3 seconds.
const TIMEOUT: Duration = Duration::from_secs(3);

/*
    Internal function which facilitate the public functions. 
*/

// Does interal things to the usb communication. Most importantly 
// open and closing the device. If you want to open, the value is 
// 2 and to close it is 4.
fn _write_to_control(handle: &mut DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(_n) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Couldn't write to control with error '{}' 'stage_control::driver::_byte_vec_to_string'", e).unwrap();
            return Err(e);
        }
    };
    return Ok(());
}


// Responses are in a vector of bytes, this converts them to a nice easy
// to read and operate on string
fn _byte_vec_to_string(byte_vec: &mut Vec<u8>) -> Result<String, std::str::Utf8Error> {
    let mut string_vec: Vec<u8> = Vec::new();
    
    for &i in byte_vec.iter() {
        if i == 0 {break;}
        string_vec.push(i);
    }
    
    return match std::str::from_utf8(&string_vec) {
        Ok(string) => Ok(string.to_string()),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Failed to convert response to string with error '{}' in 'stage_control::driver::_byte_vec_to_string'", e).unwrap();
            return Err(e);
        },
    };
}

// The device sends the command prefixed with a '?' when it was not 
// understood. This check for a question mark to relay that information
// if needed. 
fn _command_understood(response: &String) -> bool {
    if response.starts_with('?') {return false;}
    else {return true;}
}


// This clears the read buffer to make sure that there is nothing
// outstanding that may complicate further read operations. Done
// when the device is first opened and reading from bulk.
// It often "Timeout" errors if there is nothing to read so that 
// error is ignored while other errors are reported. 
fn _saftey_read(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let output: &mut Vec<u8> = &mut [0u8; 4096].to_vec();
    match handle.read_bulk(0x82, output, TIMEOUT) {
        Ok(_n) => (),
        Err(e) => match e {
            rusb::Error::Timeout => (),
            _ => writeln!(std::io::stderr(), "WARNING: Failed to read from bulk with error '{}'.", e).unwrap(),
        },
    };
    return Ok(());
}


// Cycles through all the devices connected and returns the handle for the 
// device that matches those numbers. Returns "NotFound" if there is no match
// in any of the devices. 
fn _get_handle_from_vendor_product_id(vendor_id: u16, product_id: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let devices = match devices() {
        Ok(list) => list,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Failed to get device list with error '{}' in 'stage_control::driver::_get_handle_from_vendor_product_id'.", e).unwrap();
            return Err(e);
        },
    };

    for device in devices.iter() {
        let device_desc: DeviceDescriptor = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(e) => {
                writeln!(std::io::stderr(), "ERROR: Failed to get device descriptor with error '{}' in 'stage_control::driver::_get_handle_from_vendor_product_id'.", e).unwrap();
                return Err(e);
            }
        };

        if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
            return match device.open() {
                Ok(handle) => Ok(handle),
                Err(e) => {
                    writeln!(std::io::stderr(), "ERROR: Failed to get device handle with error '{}' in 'stage_control::driver::_get_handle_from_vendor_product_id'.", e).unwrap();
                    return Err(e);
                } 
            };
        }
    }
    writeln!(std::io::stderr(), "ERROR: Failed to get find device with vendor id {} and product id {} in 'stage_control::driver::_get_handle_from_vendor_product_id'.", vendor_id, product_id).unwrap();
    return Err(rusb::Error::NotFound);
}


/*
    Public functions, read replies from the device, write commands to it, open it, close it
*/



// After sending a command to the bulk, the response is also on the bulk 
// but at a different endpoint. This reads from this endpoint and returns
// the response as a string
pub fn read_from_bulk(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let raw_output: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    let response: String = match handle.read_bulk(0x82, raw_output, TIMEOUT) {
        Ok(_n) => match _byte_vec_to_string(raw_output) {
            Ok(r) => r,
            Err(e) => {
                writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::_byte_vec_to_string' failed with error '{}' in 'stage_control::driver::read_from_bulk'", e).unwrap();
                return Err(rusb::Error::Other);
            }
        },
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Failed to read from bulk with error '{}' in 'stage_control::driver::read_from_bulk'.", e).unwrap();
            return Err(e);
        },
    };

    return match _command_understood(&response) {
        true => Ok(response),
        false => {
            writeln!(std::io::stderr(), "ERROR: Response '{}' was not understood in 'stage_control::driver::read_from_bulk'", response).unwrap();
            Err(rusb::Error::Io)
        }
    };
}

// Writing to bulk is how to actually send a command to the device. This  
// must be through a byte array of the ascii characters. Command MUST be 
// terminated by a null character '\0' otherwise you'll have a bad time.
pub fn write_to_bulk(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written: usize = match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => n,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: failed to write to bulk with error '{}' in 'stage_control::driver::write_to_bulk'", e).unwrap();
            return Err(rusb::Error::Other);
        },
    };

    if bytes_written != command.len() {
        writeln!(std::io::stderr(), "ERROR: Command failed to properly send in 'stage_control::driver::write_to_bulk'. Incorrect number of bytes written").unwrap();
        return Err(rusb::Error::Io);
    }
    return Ok(());
}

// Opens the device and returns the handle. This involves: finding the device,
// claiming the interface, writing to control, and then clearing anything
// outstanding in the buffer. From there, it should be primed and ready to go
pub fn open(vendor_id: u16, product_id: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let mut handle: DeviceHandle<GlobalContext> = match _get_handle_from_vendor_product_id(vendor_id, product_id) {
        Ok(handle) => handle,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::_get_handle_from_vendor_product_id' failed with error '{}' in 'stage_control::driver::open'", e).unwrap();
            return Err(e);
        }
    };

    match handle.claim_interface(0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Failed to claim interface with error '{}' in 'stage_control::driver::open'", e).unwrap();
            return Err(e);
        },
    };

    match _write_to_control(&mut handle, 2) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::_write_to_control' failed with error '{}' in 'stage_control::driver::open'", e).unwrap();
            return Err(e)
        },
    };

    match _saftey_read(&mut handle) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::_saftey_read' with error '{}' in 'stage_control::driver::open'", e).unwrap();
            return Err(e);
        },
    };

    return Ok(handle);
}


// Closes device and releases interface. Releaseing the interface may be
// important if communicaiton across threads is created
pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match _write_to_control(handle, 4) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "WARNING: Function 'stage_control::driver::_write_to_control' failed with error '{}' in 'stage_control::driver::close'", e).unwrap(),
    };

    match handle.release_interface(0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "WARNING: Failed to release interface with error '{}' in 'stage_control::driver::close'", e).unwrap();
            return Err(e);
        },
    };
    return Ok(());
}



// Combines write_to_bulk and read_from_bulk into one convienent function. 
pub fn send_command_get_response(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<String> {
    match _saftey_read(handle) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "WARNING: Function 'stage_control::driver::_saftey_read' failed with error '{}' in 'stage_control::driver::send_command_get_response'", e).unwrap(),
    };
    
    match write_to_bulk(handle, command) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::write_to_bulk' failed with error '{}' in 'stage_control::driver::send_command_get_response'", e).unwrap();
            return Err(e);
        },
    };

    return match read_from_bulk(handle) {
        Ok(r) => Ok(r),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::driver::read_from_bulk' failed with error '{}' in 'stage_control::driver::send_command_get_response'", e).unwrap();
            return Err(e);
        },
    };
}






