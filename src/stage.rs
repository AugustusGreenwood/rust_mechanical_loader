use std::io::Read;

use rusb::{
    DeviceHandle, devices, GlobalContext, DeviceDescriptor
};

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);


// Begin functions

pub fn write_to_control(handle: &mut DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(_n) => (),
        Err(e) => {
            println!("%%%% Couldn't write to control buffer.\n%%%% Non-critical error: \"{}\"\n", e);
            return Err(e);
        },
    };
    Ok(())
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
        Ok(()) => (),
        
        Err(e) => {
            println!("%%%% Couldn't claim interface!\n%%%% Critical error\n%%%% Exiting\n");
            return Err(e);
        },
    };

    write_to_control(&mut handle, 2)?;
    
    match handle.read_bulk(0x82, &mut [0; 4096], TIMEOUT) {
        Ok(_n) => (),
        
        Err(e) => println!("%%%% Couldn't clear read buffer.\n%%%% Non-critical error: \"{}\"\n", e),
    };

    turn_motor_on(&mut handle)?;

    Ok(handle)
}

pub fn extract_response_from_raw_output(output: &mut Vec<u8>) -> String {
    let mut response: Vec<u8> = Vec::new();
    for &i in output.iter() {
        if i == 0 {break;}
        response.push(i);
    }
    std::str::from_utf8(&response).unwrap().to_string()
}

pub fn write_to_bulk(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written: usize = match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => n,
        
        Err(e) => {
            println!("Couldn't bulk write with error: {}\nExiting to be safe\n", e);
            return Err(e);
        },
    };

    if bytes_written != command.len() {
        println!("Incorrent number of bytes written. Command was likely not sent properly. {} bytes written when {} were expected
                with command {:#?}.", bytes_written, command.len(), command);
    }

    Ok(())
}

pub fn read_from_bulk(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<String> {
    let output: &mut Vec<u8> = &mut [0u8; 64].to_vec();
    let response: String = match handle.read_bulk(0x82, output, TIMEOUT) {
        Ok(_n) => extract_response_from_raw_output(output),
        Err(e) => {
            println!("Couldn't bulk read with error{:#?}\n", e);
            return Err(e);
        },
    };  
    Ok(response)
}

pub fn send_command_get_response(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<String> {
    write_to_bulk(handle, command)?;

    let response: String = read_from_bulk(handle)?;

    Ok(response)
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    write_to_control(handle, 4)?;
    
    match handle.release_interface(0) {
        Ok(()) => (),
        Err(e) => {
            println!("Couldn't release interface! Possibly bad, you might need to restart the device\n");
            return Err(e);
        },
    };
    Ok(())
}

pub fn check_for_valid_response(response: &String, error_log: &str) -> rusb::Result<()> {
    if response.starts_with('?') {
        println!("Response {} is invalid.\n{}", response, error_log);
        return Err(rusb::Error::Io);
    }
    Ok(())
}

// Begin functions



// Other

pub fn turn_motor_on(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't turn motor on";
    match send_command_get_response(handle, "EO=1\0".as_bytes()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(_e) => panic!("Couldn't turn motor on, exiting to avoid mechanical errors"),
    };
    Ok(())
}

pub fn turn_motor_off(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't turn motor off";
    match send_command_get_response(handle, "EO=0\0".as_bytes()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => panic!("Couldn't turn motor off due to error {}, exiting to avoid mechanical errors", e),
    };
    Ok(())
}

pub fn write_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't write driver settings";
    match send_command_get_response(handle, "RW\0".as_bytes()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };

    turn_motor_on(handle)?;
    Ok(())
}

pub fn update_readable_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't read driver settings";
    match send_command_get_response(handle, "RR\0".as_bytes()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    println!("WARNING: Motor is off\n");
    Ok(())
}

// Other


// Getters

pub fn get_high_speed(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "HSPD\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),
        
        Err(e) => {
            println!("Couldn't get high speed");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_low_speed(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "LSPD\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),
        
        Err(e) => {
            println!("Couldn't get low speed");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_acceleration_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "ACC\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),

        Err(e) => {
            println!("Couldn't get acceleration time");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_deceleration_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "DEC\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),
        
        Err(e) => {
            println!( "Couldn't get deceleration time");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_pulse_position(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "PX\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),
        
        Err(e) => {
            println!("Couldn't get pulse position");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn acceleration_profile_is_sin(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "SCV\0".as_bytes()) {
        Ok(response) => response.parse().unwrap(),
        
        Err(e) => {
            println!("Couldn't get acceleration profile");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_idle_time(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    update_readable_driver_settings(handle)?;
    
    let response: i32 = match send_command_get_response(handle, "DRVIT\0".as_bytes()) {
        Ok(response) => response.parse().unwrap(),
        
        Err(e) => {
            println!("Couldn't get idle time");
            return Err(e);
        }
    };
    turn_motor_on(handle)?;
    Ok(response)
}

// Getters



// Setters

pub fn set_high_speed(handle: &mut DeviceHandle<GlobalContext>, new_high_speed: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set high speed";
    match send_command_get_response(handle, &["HSPD=".as_bytes(), &new_high_speed.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_low_speed(handle: &mut DeviceHandle<GlobalContext>, new_low_speed: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set low speed";
    match send_command_get_response(handle, &["LSPD=".as_bytes(), &new_low_speed.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_acceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_acceleration_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set acceleration time";
    match send_command_get_response(handle, &["ACC=".as_bytes(), &new_acceleration_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_deceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_deceleration_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set deceleration time";
    match send_command_get_response(handle, &["DEC=".as_bytes(), &new_deceleration_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_pulse_position(handle: &mut DeviceHandle<GlobalContext>, pulse_position: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set pulse position";
    match send_command_get_response(handle, &["PX".as_bytes(), &pulse_position.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_baud_rate(handle: &mut DeviceHandle<GlobalContext>, baud_rate: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set baud rate";
    match send_command_get_response(handle, &["DB=".as_bytes(), &baud_rate.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_idle_time(handle: &mut DeviceHandle<GlobalContext>, idle_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set idle time";
    match send_command_get_response(handle, &["DRVIT=".as_bytes(), &idle_time.to_le_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => check_for_valid_response(&response, error_log)?,

        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    write_driver_settings(handle)?;
    Ok(())
}

// Setters


pub fn interactive_mode(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    loop {
        let mut command = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .expect("Failed to read line");
        
        if command.trim() == "EXIT" {
            break;
        }

        let response = send_command_get_response(handle, command.as_bytes())?;
        println!("{}", response);
    }
    Ok(())
}