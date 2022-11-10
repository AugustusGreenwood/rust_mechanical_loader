use rusb::{
    DeviceHandle, devices, GlobalContext, DeviceDescriptor
};

pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);



/* Interanl functions */

fn _write_to_control(handle: &mut DeviceHandle<GlobalContext>, value: u16) -> rusb::Result<()> {
    match handle.write_control(64, 2, value, 0, &[], TIMEOUT) {
        Ok(_n) => (),
        Err(e) => {
            println!("%%%% Couldn't write to control buffer.\n%%%% Non-critical error: \"{}\"", e);
            return Err(e);
        },
    };
    Ok(())
}

fn _check_for_valid_response(response: &String, error_log: &str) -> rusb::Result<()> {
    if response.starts_with('?') {
        println!("Response {} is invalid.\n{}", response, error_log);
        return Err(rusb::Error::Io);
    }
    Ok(())
}

fn _extract_response_from_raw_output(output: &mut Vec<u8>) -> String {
    let mut response: Vec<u8> = Vec::new();
    for &i in output.iter() {
        if i == 0 {break;}
        response.push(i);
    }
    std::str::from_utf8(&response).unwrap().to_string()
}

fn _saftey_read(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let output: &mut Vec<u8> = &mut [0u8; 4096].to_vec();
    match handle.read_bulk(0x82, output, std::time::Duration::from_millis(1)) {
        Ok(_n) => (),
        Err(e) => match e {
            rusb::Error::Timeout => (),
            _ => println!("Can't saftey read with error: {}", e),
        },
    };  
    Ok(())
}


/* Open (return handle to device) and close device, respectively */

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
            println!("Couldn't find device, make sure it is on and plugged in.");
            return Err(rusb::Error::NotFound);
        },
    };

    match handle.claim_interface(0) {
        Ok(()) => (),
        
        Err(e) => {
            println!("Couldn't claim interface! Cannot interact with device without a claimed interface.");
            return Err(e);
        },
    };

    _write_to_control(&mut handle, 2)?;
    
    _saftey_read(&mut handle)?;

    Ok(handle)
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    _write_to_control(handle, 4)?;
    
    match handle.release_interface(0) {
        Ok(()) => (),
        Err(e) => {
            println!("Couldn't release interface! Possibly bad, you might need to restart the device");
            return Err(e);
        },
    };
    Ok(())
}


/* 
    Function which are mainly used in the rest of the script when dealing with commands. ie they write to the usb, 
    read from the usb, or they impletment them both in a package to send a command to the usb and then get the following response 
*/

pub fn write_to_bulk(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<()> {
    let bytes_written: usize = match handle.write_bulk(0x02, command, TIMEOUT) {
        Ok(n) => n,
        
        Err(e) => {
            println!("Couldn't bulk write with error: {}\nExiting to be safe", e);
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
        Ok(_n) => _extract_response_from_raw_output(output),
        Err(e) => {
            println!("Couldn't bulk read with error: {:#?}", e);
            return Err(e);
        },
    };  
    Ok(response)
}

pub fn send_command_get_response(handle: &mut DeviceHandle<GlobalContext>, command: &[u8]) -> rusb::Result<String> {
    _saftey_read(handle)?;
    
    write_to_bulk(handle, command)?;

    let response: String = read_from_bulk(handle)?;

    Ok(response)
}




/* 
    Functions which don't set anything or get anything (technically). They either deal with the state of the motor
    or write updated driver settings, or update the readable driver settings.
*/

pub fn turn_motor_on(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't turn motor on";
    match send_command_get_response(handle, "EO=1\0".as_bytes()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(_e) => panic!("Couldn't turn motor on, exiting to avoid mechanical errors"),
    };
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}

pub fn turn_motor_off(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't turn motor off";
    match send_command_get_response(handle, "EO=0\0".as_bytes()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => panic!("Couldn't turn motor off due to error {}, exiting to avoid mechanical errors", e),
    };
    std::thread::sleep(std::time::Duration::from_secs(3));
    Ok(())
}

pub fn write_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't write driver settings";
    match send_command_get_response(handle, "RW\0".as_bytes()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(2500));
    Ok(())
}

pub fn update_readable_driver_settings(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let error_log: &str = "Couldn't read driver settings";
    match send_command_get_response(handle, "RR\0".as_bytes()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(2500));
    Ok(())
}



/* 
    All the function which get some parameter of the device. They send the relevent command
    parse the output into an intereger and return an integer. I don't have these check for a
    valid output since the parse will fail if the command was not understood.
*/

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
    let response: i32 = match send_command_get_response(handle, "DRVIT\0".as_bytes()) {
        Ok(response) => response.parse().unwrap(),
        
        Err(e) => {
            println!("Couldn't get idle time");
            return Err(e);
        }
    };
    Ok(response)
}

pub fn get_motor_status(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "MST\0".as_bytes()) {
        Ok(response) => response.parse().unwrap(),
        
        Err(e) => {
            println!("Couldn't get pulse position");
            return Err(e);
        }
    };
    Ok(response)
}




/* 
    These function set some value in the device. With these, I check the output for a "?" since that is how
    performax communicates it didn't understand a command. I use this to just tell the user that the command
    wasn't understood and whatever they expected to be set, wasn't set. 
*/

pub fn set_high_speed(handle: &mut DeviceHandle<GlobalContext>, new_high_speed: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set high speed";
    match send_command_get_response(handle, &["HSPD=".as_bytes(), &new_high_speed.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_low_speed(handle: &mut DeviceHandle<GlobalContext>, new_low_speed: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set low speed";
    match send_command_get_response(handle, &["LSPD=".as_bytes(), &new_low_speed.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_acceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_acceleration_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set acceleration time";
    match send_command_get_response(handle, &["ACC=".as_bytes(), &new_acceleration_time.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_deceleration_time(handle: &mut DeviceHandle<GlobalContext>, new_deceleration_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set deceleration time";
    match send_command_get_response(handle, &["DEC=".as_bytes(), &new_deceleration_time.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_pulse_position(handle: &mut DeviceHandle<GlobalContext>, pulse_position: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set pulse position";
    match send_command_get_response(handle, &["PX=".as_bytes(), &pulse_position.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,
        
        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_idle_time(handle: &mut DeviceHandle<GlobalContext>, idle_time: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set idle time";
    match send_command_get_response(handle, &["DRVIT=".as_bytes(), &idle_time.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,

        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(())
}

pub fn set_movement_type(handle: &mut DeviceHandle<GlobalContext>, abs_inc: &str) -> rusb::Result<()> {
    match abs_inc {
        "abs" => send_command_get_response(handle, "ABS\0".as_bytes())?,
        "ABS" => send_command_get_response(handle, "ABS\0".as_bytes())?,

        "inc" => send_command_get_response(handle, "INC\0".as_bytes())?,
        "INC" => send_command_get_response(handle, "INC\0".as_bytes())?,

        _ => {
            println!("Couldn't understand absolute or increment command. Use 'INC', 'ABS', 'inc' or 'abs'");
            return Err(rusb::Error::Other);
        },
    };
    Ok(())
}

pub fn set_microsteps(handle: &mut DeviceHandle<GlobalContext>, microsteps: i32) -> rusb::Result<()> {
    let error_log: &str = "Couldn't set microsteps";
    match send_command_get_response(handle, &["DRVMS=".as_bytes(), &microsteps.to_string().as_bytes(), "\0".as_bytes()].concat()) {
        Ok(response) => _check_for_valid_response(&response, error_log)?,

        Err(e) => {
            println!("{}", error_log);
            return Err(e);
        }
    };
    Ok(()) 
}


/* 
    This all are usually involve movement of some kind of advancement settings or just do more than simply set/get 
*/
pub fn move_stage(handle: &mut DeviceHandle<GlobalContext>, abs_inc: &str, distance: i32) -> rusb::Result<()> {
    set_movement_type(handle, abs_inc)?;
    send_command_get_response(handle, &["X".as_bytes(), &distance.to_string().as_bytes(), "\0".as_bytes()].concat())?;
    Ok(())
}

pub fn interactive_mode(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    println!("Entering interactive mode");
    loop {
        let mut raw_command = String::new();
        std::io::stdin()
            .read_line(&mut raw_command)
            .expect("Failed to read line");
        
        let command = raw_command.trim().to_ascii_uppercase();

        if command == "EXIT" || command == "E" {
            break;
        }

        let response = send_command_get_response(handle, &[command.as_bytes(), "\0".as_bytes()].concat())?;
        println!("{}", response);
    }
    Ok(())
}

pub fn wait_for_motor_idle(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    loop {
        if get_motor_status(handle)? == 0 {
            return Ok(())
        }
    }
}

pub fn move_one_cycle(handle: &mut DeviceHandle<GlobalContext>, distance: i32) -> rusb::Result<()> {
    move_stage(handle, "INC", distance)?;
    wait_for_motor_idle(handle)?;
    move_stage(handle, "INC", -distance)?;
    Ok(())
}