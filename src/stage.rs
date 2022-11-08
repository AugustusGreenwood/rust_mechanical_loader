use rusb::{
    DeviceHandle, devices, GlobalContext, DeviceDescriptor
};

const STAGERS_DEBUG: bool = true;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);


// Deiver functions

pub fn write_to_control(handle: &mut DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<usize> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(n) => {
            if STAGERS_DEBUG == true {
                println!("Function \"write_to_control()\" in file \"stage.rs\" using \"handle.write_control\" succesfully wrote {} bytes. 0 is expected\n", n);
            }
            return Ok(n);
        },
        Err(e) => {
            println!("%%%% Couldn't write to control buffer.\n%%%% Non-critical error: \"{}\"\n", e);
            return Err(e);
        },
    };
}

pub fn open() -> rusb::Result<DeviceHandle<GlobalContext>> {
    let mut handle__: Option<DeviceHandle<GlobalContext>> = None;

    for device in devices()?.iter() {
        let device_desc: DeviceDescriptor = device.device_descriptor()?;

        if device_desc.vendor_id() == 0x1589 && device_desc.product_id() == 0xa101 {
            handle__ = Some(device.open()?);
            break;
        }
    }

    let mut handle: DeviceHandle<GlobalContext> = match handle__ {
        Some(handle) => handle,
        None => {
            println!("Couldn't find device, make sure it is on and plugged in.\n");
            return Err(rusb::Error::NotFound);
        },
    };

    match handle.claim_interface(0) {
        Ok(()) => {
            if STAGERS_DEBUG == true {
                println!("Device interface successfully claimed\n");
            }
        },
        Err(e) => {
            println!("%%%% Couldn't claim interface!\n%%%% Critical error\n%%%% Exiting\n");
            return Err(e);
        },
    };

    write_to_control(&mut handle, 2)?;
    
    match handle.read_bulk(0x82, &mut [0; 4096], TIMEOUT) {
        Ok(n) => {
            if STAGERS_DEBUG == true {
                println!("Function \"open()\" in file \"stage.rs\" using \"handle.read_bulk\" succesfully read {} bytes. 0 is expected\n", n);
            }
        },
        Err(e) => println!("%%%% Couldn't clear read buffer.\n%%%% Non-critical error: \"{}\"\n", e),
    };

    Ok(handle)
}

pub fn extract_response(response: &mut Vec<u8>) -> String {
    let mut extracted: Vec<u8> = Vec::new();

    for &i in response.iter() {
        if i == 0 {
            break;
        }
        extracted.push(i);
    }
    std::str::from_utf8(&extracted).unwrap().to_string()
}

pub fn write_to_bulk(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => {
            if STAGERS_DEBUG == true {
                println!("Function \"send_command()\" in file \"stage.rs\" using \"handle.write_bulk\" succesfully 
                wrote {} bytes, with command \"{}\", with timeout {:#?}, at endpoint {}\n", n, std::str::from_utf8(command).unwrap(), TIMEOUT, 0x02);
            }
        },
        Err(e) => {
            println!("Couldn't bulk write with error: {}\nExiting to be safe\n", e);
            return Err(e);
        },
    };
    Ok(())
}

pub fn read_from_bulk(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let response: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    match handle.read_bulk(0x82, response, TIMEOUT) {
        Ok(n) => {
            if STAGERS_DEBUG == true {
                println!("Function \"send_command()\" in file \"stage.rs\" using \"handle.read_bulk\" succesfully 
                read {} bytes, recieved buffer: \"{}\" with timeout {:#?}, at endpoint {}\n", n, extract_response(response), TIMEOUT, 0x82);
            }
        },
        Err(e) => {
            println!("Couldn't bulk read with error{:#?}\n", e);
            return Err(e);
        },
    };  

    Ok(extract_response(response))
    let extracted_response: String = extract_response(response);
    if extracted_response.starts_with('?') {
        println!("Command {:#?} not understand with response {:#?}", command, extracted_response);
        return Err(rusb::Error::Io);
    }
}

pub fn send_command_get_response(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<String> {
    write_to_bulk(handle, command)?;

    let response: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    match handle.read_bulk(0x82, response, TIMEOUT) {
        Ok(n) => {
            if STAGERS_DEBUG == true {
                println!("Function \"send_command()\" in file \"stage.rs\" using \"handle.read_bulk\" succesfully 
                read {} bytes, recieved buffer: \"{}\" with timeout {:#?}, at endpoint {}\n", n, extract_response(response), TIMEOUT, 0x82);
            }
        },
        Err(e) => {
            println!("error {:#?}\n", e);
        },
    };

    let extracted_response: String = extract_response(response);
    if extracted_response.starts_with('?') {
        println!("Command {:#?} not understand with response {:#?}", command, extracted_response);
        return Err(rusb::Error::Io);
    }
    Ok(extract_response(response))
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    write_to_control(handle, 4)?;
    
    match handle.release_interface(0) {
        Ok(()) => {
            if STAGERS_DEBUG == true {
                println!("Device interface successfully released\n");
            }
            return Ok(());
        },
        Err(e) => {
            println!("Couldn't release interface! Possibly bad, you might need to restart the device\n");
            return Err(e);
        },
    };
}

// Driver functions



// Other

pub fn turn_motor_on(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let command: &[u8] = "EO=1\0".as_bytes();
    match send_command_get_response(handle, command) {
        Ok(response) => {
            if response.starts_with('?') {
                panic!("Couldn't understand command {:#?}, exiting to avoid mechanical errors", command);
            }
            else {return Ok(());}
        },
        Err(_e) => {
            panic!("Couldn't turn motor on, exiting to avoid mechanical errors");
        }
    };
}

pub fn turn_motor_off(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let command: &[u8] = "EO=0\0".as_bytes();
    match send_command_get_response(handle, command) {
        Ok(response) => {
            if response.starts_with('?') {
                panic!("Couldn't understand command {:#?}, exiting to avoid mechanical errors", command);
            }
            else {return Ok(());}
        },
        Err(_e) => {
            panic!("Couldn't turn motor off, exiting to avoid mechanical errors");
        }
    };
}

pub fn write_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match send_command_get_response(handle, "RW\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't write driver settings");
                return Err(rusb::Error::Io);
            }
            else {
                turn_motor_on(handle)?;
                return Ok(());
            }
        },
        Err(e) => {
            println!("Couldn't write driver settings");
            return Err(e);
        }
    };
}

pub fn read_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match send_command_get_response(handle, "RR\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't read driver settings");
                return Err(rusb::Error::Io);
            }
            else {
                println!("WARNING: Motor is off\n");
                return Ok(());
            }
        },
        Err(e) => {
            println!("Couldn't read driver settings");
            return Err(e);
        }
    };
}

// Other


// Getters

pub fn get_high_speed(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "HSPD\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get high speed");
                return Err(rusb::Error::Io);
            }
            else {return Ok(response.parse::<i32>().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get high speed");
            return Err(e);
        }
    };
}

pub fn get_low_speed(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "LSPD\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get low speed");
                return Err(rusb::Error::Io);
            }
            else {return Ok(response.parse::<i32>().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get low speed");
            return Err(e);
        }
    };
}

pub fn get_acceleration_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "ACC\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get acceleration time");
                return Err(rusb::Error::Io);
            }
            else {return Ok(response.parse::<i32>().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get acceleration time");
            return Err(e);
        }
    };
}

pub fn get_deceleration_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "DEC\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get deceleration time");
                return Err(rusb::Error::Io);
            }
            else{return Ok(response.parse::<i32>().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get deceleration time");
            return Err(e);
        }
    };
}

pub fn get_pulse_position(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "PX\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get pulse position");
                return Err(rusb::Error::Io);
            }
            else{return Ok(response.parse::<i32>().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get pulse position");
            return Err(e);
        }
    };
}

pub fn acceleration_profile_is_sin(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    match send_command_get_response(handle, "SCV\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't get acceleration profile");
                return Err(rusb::Error::Io);
            }
            else {return Ok(response.parse().unwrap());}
        },
        Err(e) => {
            println!("Couldn't get acceleration profile");
            return Err(e);
        }
    };
}

pub fn get_idle_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    read_driver_settings(handle)?;
    match send_command_get_response(handle, "DRVIT\0".as_bytes()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set idle time");
                return Err(rusb::Error::Io);
            }
            else {
                turn_motor_on(handle)?;
                return Ok(response.parse().unwrap());
            }
        },
        Err(e) => {
            println!("Couldn't set idle time");
            return Err(e);
        }
    };
}

// Getters



// Setters

pub fn set_high_speed(handle: &mut DeviceHandle<GlobalContext>, new_high_speed: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["HSPD=".as_bytes(), &new_high_speed.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't get high speed");
            return Err(e);
        }
    };
}

pub fn set_low_speed(handle: &mut DeviceHandle<GlobalContext>, new_low_speed: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["LSPD=".as_bytes(), &new_low_speed.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set low speed");
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't set low speed");
            return Err(e);
        }
    };
}

pub fn set_acceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_acceleration_time: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["ACC=".as_bytes(), &new_acceleration_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set acceleration time");
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't set acceleration time");
            return Err(e);
        }
    };
}

pub fn set_deceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_deceleration_time: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["DEC=".as_bytes(), &new_deceleration_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set deceleration time");
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't set deceleration time");
            return Err(e);
        }
    };
}

pub fn set_pulse_position(handle: &mut DeviceHandle<GlobalContext>, pulse_position: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["PX".as_bytes(), &pulse_position.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set deceleration time");
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't set deceleration time");
            return Err(e);
        }
    };
}

pub fn set_baud_rate(handle: &mut DeviceHandle<GlobalContext>, baud_rate: i32) -> rusb::Result<bool> {
    match send_command_get_response(handle, &["DB=".as_bytes(), &baud_rate.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set baud rate");
                return Ok(false);
            }
            else {return Ok(true);}
        },
        Err(e) => {
            println!("Couldn't set baud rate");
            return Err(e);
        }
    };
}

pub fn set_idle_time(handle: &mut DeviceHandle<GlobalContext>, idle_time: i32) -> rusb::Result<()> {
    match send_command_get_response(handle, &["DRVIT=".as_bytes(), &idle_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => {
            if response.starts_with('?') {
                println!("Couldn't set idle time");
                return Err(rusb::Error::Io);
            }
            else {
                write_driver_settings(handle)?;
                return Ok(());
            }
        },
        Err(e) => {
            println!("Couldn't set idle time");
            return Err(e);
        }
    };
}

// Setters


