use crate::stage_control::driver::{
    get_handle_from_vendor_product_id, read_from_bulk, saftey_read, write_to_bulk, write_to_control,
};

use rusb::{DeviceHandle, GlobalContext, Result};

use std::{
    fs::File,
    io::{stdin, BufWriter, Write},
    time::{Duration, Instant},
};

pub fn open(vendor_id: u16, product_id: u16) -> Result<DeviceHandle<GlobalContext>> {
    let mut handle = get_handle_from_vendor_product_id(vendor_id, product_id)?;
    handle.claim_interface(0)?;
    write_to_control(&handle, 2)?;

    saftey_read(&handle)?;
    Ok(handle)
}

// Notice we don't release the interface, rusb does that automatically when the
// variable goes out of scope and it means we don't need the handle as mutable
pub fn close(handle: &DeviceHandle<GlobalContext>) -> Result<()> {
    write_to_control(handle, 4)?;
    Ok(())
}

pub fn send_command_get_response(
    handle: &DeviceHandle<GlobalContext>,
    command: &[u8],
) -> Result<String> {
    saftey_read(handle)?;
    write_to_bulk(handle, command)?;
    read_from_bulk(handle)
}

pub fn write_driver_settings(handle: &DeviceHandle<GlobalContext>) -> Result<()> {
    let _ = send_command_get_response(handle, b"RW\0")?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    check_driver_write(handle)?;
    Ok(())
}

pub fn check_driver_write(handle: &DeviceHandle<GlobalContext>) -> Result<()> {
    let response = send_command_get_response(handle, b"R4\0")?;
    // This should probably error in a more breaking way. Bad driver writes can be bad
    if response != "1" {
        writeln!(
            std::io::stderr(),
            "WARNING: Driver write failed, values may not be set, device responded '{}'",
            response
        )
        .unwrap();
        return Err(rusb::Error::Other);
    }
    Ok(())
}

// Serious question, should this be unsigned? A negative high speed is not understood,
// but this requires some casting later on. It seems safer to do this, but will see.
pub fn set_high_speed(handle: &DeviceHandle<GlobalContext>, new_high_speed: u32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"HSPD=", new_high_speed.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn move_stage(handle: &DeviceHandle<GlobalContext>, position: i32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"X", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_low_speed(handle: &DeviceHandle<GlobalContext>, new_low_speed: u32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"LSPD=", new_low_speed.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_acceleration_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"ACC=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_acceleration_profile(
    handle: &DeviceHandle<GlobalContext>,
    sin_trap: &str,
) -> Result<()> {
    let command = match sin_trap {
        "sin" => b"SCV=1\0",
        "trap" => b"SCV=0\0",
        _ => return Err(rusb::Error::Other),
    };

    let _ = send_command_get_response(handle, command)?;
    Ok(())
}

pub fn set_deceleration_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"DEC=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_idle_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> Result<()> {
    if time < 1 || time > 100 {
        return Err(rusb::Error::InvalidParam);
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVIT=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn turn_motor_on(handle: &DeviceHandle<GlobalContext>) -> Result<()> {
    let _ = send_command_get_response(handle, b"EO=1\0")?;
    std::thread::sleep(Duration::from_secs(3));
    Ok(())
}

pub fn set_microstepping(handle: &DeviceHandle<GlobalContext>, microsteps: u32) -> Result<()> {
    if microsteps < 2 || microsteps > 500 {
        return Err(rusb::Error::InvalidParam); // TODO Certainly there is a better error than this
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVMS=", microsteps.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_idle_current(handle: &DeviceHandle<GlobalContext>, current: u32) -> Result<()> {
    if current < 100 || current > 2800 {
        return Err(rusb::Error::InvalidParam); // TODO Certainly there is a better error than this
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVIC=", current.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_run_current(handle: &DeviceHandle<GlobalContext>, current: u32) -> Result<()> {
    if current < 100 || current > 3000 {
        return Err(rusb::Error::InvalidParam); // TODO Certainly there is a better error than this
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVRC=", current.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_movement_type(handle: &DeviceHandle<GlobalContext>, abs_inc: &str) -> Result<()> {
    let command = match abs_inc.to_ascii_lowercase().as_str() {
        "abs" => b"ABS\0",
        "inc" => b"INC\0",
        _ => return Err(rusb::Error::InvalidParam),
    };

    let _ = send_command_get_response(handle, command)?;
    Ok(())
}

pub fn get_high_speed(handle: &DeviceHandle<GlobalContext>) -> Result<u32> {
    let response: u32 = send_command_get_response(handle, b"HSPD\0")?
        .parse()
        .unwrap();
    Ok(response)
}

pub fn set_pulse_position(handle: &DeviceHandle<GlobalContext>, position: i32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"PX=", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn set_encoder_position(handle: &DeviceHandle<GlobalContext>, position: i32) -> Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"EX=", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    Ok(())
}

pub fn get_pulse_position(handle: &DeviceHandle<GlobalContext>) -> Result<i32> {
    let response: i32 = send_command_get_response(handle, b"PX\0")?.parse().unwrap();
    Ok(response)
}

pub fn get_encoder_position(handle: &DeviceHandle<GlobalContext>) -> Result<i32> {
    let response: i32 = send_command_get_response(handle, b"EX\0")?.parse().unwrap();
    Ok(response)
}

pub fn get_motor_status(handle: &DeviceHandle<GlobalContext>) -> Result<i32> {
    let response: i32 = send_command_get_response(handle, b"MST\0")?
        .parse()
        .unwrap();
    Ok(response)
}

pub fn output_time_pos_to_file(
    handle: &DeviceHandle<GlobalContext>,
    file: &mut BufWriter<File>,
    time: Instant,
) -> Result<()> {
    file.write_all(
        &[
            time.elapsed().as_secs_f64().to_string().as_bytes(),
            b"\t",
            get_pulse_position(handle)?.to_string().as_bytes(),
            b"\n",
        ]
        .concat(),
    )
    .unwrap();
    Ok(())
}

pub fn wait_for_motor_idle(
    handle: &DeviceHandle<GlobalContext>,
    file: &mut Option<BufWriter<File>>,
    time: Option<Instant>,
) -> Result<()> {
    match file {
        None => while get_motor_status(handle)? != 0 {},
        Some(file) => {
            while get_motor_status(handle)? != 0 {
                output_time_pos_to_file(handle, file, time.unwrap())?;
            }
            file.flush().unwrap();
        }
    }
    Ok(())
}

pub fn move_cycle_get_time(
    handle: &DeviceHandle<GlobalContext>,
    distance: i32,
    file: &mut Option<BufWriter<File>>,
    time: Option<Instant>,
    dwell: f64,
) -> Result<f64> {
    let cycle_time = Instant::now();
    move_stage(handle, -distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(Duration::from_secs_f64(dwell));
    move_stage(handle, distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(Duration::from_secs_f64(dwell));
    Ok(cycle_time.elapsed().as_secs_f64())
}

pub fn move_cycle(handle: &DeviceHandle<GlobalContext>, distance: i32, dwell: f64) -> Result<()> {
    move_stage(handle, -distance)?;
    wait_for_motor_idle(handle, &mut None, None)?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell));
    move_stage(handle, distance)?;
    wait_for_motor_idle(handle, &mut None, None)?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell));
    Ok(())
}

fn _print_interactive_mode_help_message() {
    println!(
        "
        DESCRIPTION:
            You are currently in interactive mode. In this mode you can send commands which are
            provided by the Newark stepper motor control box. These controls have two main structures:
            simply 'COMMAND' and 'VAR=###'. The two methods are to control the device, and set variables
            which control how it moves. 

            For example, if I want to move the device, I can send the command: J+ which will move the 
            motor up indefinitely. If I want to control how quickly it moves, I would write: HSPD=1000
            and it will move at a speed of 1000 pulses/second. Importantly, the device expects all caps
            but the program automatically capitalizes all words so commands can be sent as either lower 
            or upper case. ie hspd=1000 and HSPD=1000 will both work fine.

            'J+' and 'J-' will move until a 'STOP' command is submitted. I want to implement a saftey 
            percaution that automatically stops movement after some time. For now, it is the users job
            to make sure they monitor the device while it is moving.

            There are special driver parameters, they are the idle and run current, idle time and microsteps.
            If you want to either read, or write these variables you have to enter special commands that either
            actually to the writting or reading. Additionally, driver reads and writes can fail. To tell if it 
            failed there are other command that can be submitted to check. Run DW to write driver parameters and 
            R4 to check the write. 1 means it was succesful, all else means it failed. Run RR to read driver 
            parameters, R2 to check the read. 1 means it was successfull, all else means it failed. An example 
            of how to first read a driver parameter, and change it is below in EXAMPLES section.

            The following Return Values sections shows what a command may return and what it means

            There are many other commands and messages in the manual but these are the most important and useful

        COMMANDS:
            EXIT        Exit interactive mode 
            STOP        Stop motor movement

            X####       Move to pulse position ####
            X           Get current pulse position

            J+          Move motor up until a stop command is issued
            J-          Move motor down until a stop command is issued

            ACC=###     Set acceleration value to ###
            ACC         Get current acceleration value 

            HSPD=###    Set high speed
            HSPD        Get current high speed

            LSPD=###    Set low speed
            LSPD        Get current low speed

            INC         Set movement mode to incremental
            ABS         Set movement mode to absolute

            EO=1        Turn motor on 
            EO=0        Turn motor off
            EO          Get motor status (1=on, 0=off)

            EX=###      Set encoder position 
            EX          Get encoder position

            MST         Get motor status (0=motor running, 1=motor acceleration, 2=motor decelerating)

            PX=###      Set current pulse position 
            PX          Get current pulse position

            DRVIC=[100-2800]mA  Set idle current (DRIVER PARAMETER)
            DRVIC               Get idle current (DRIVER PARAMETER)

            DRVRC=[100-3000]mA  Set running current (DRIVER PARAMETER) 
            DRVRC               Get running current (DRIVER PARAMETER) 

            DRVIT=[1-100]cs     Set idle time (DRIVER PARAMETER) 
            DRVIT               Get idle time (DRIVER PARAMETER) 

            DRVMS=[2-500]       Set microsteps (DRIVER PARAMETER) 
            DRVMS               Get microsteps (DRIVER PARAMETER) 


        RETURN VALUES:
            ?[command]  -> Command was not understood 
            ?Moving     -> A move or position change was sent while the motor is moving 
        
        EXAMPLES:
            Read and write driver parameter:
                RR ; update all driver parameters, wait 2 seconds
                    ; Returns OK    this means the command was understood, not that the read was sucessfull
                R2 ; Make sure the read was succesful, wait 2 seconds
                    ; Returns 1     Successful
                DRVIT ; Check what the current idle time is 
                    ; Returns 100   Idles 100 cs before saying it's done moving
                DRVIT=200 ; Increase the idle time to 200 cs 
                    ; Returns OK    Again, the command was understood, but the motor hasn't actually written the value 
                RW  ; Actaully writes the 200cs value to DRVIT, wait 2 seconds
                    ; Returns OK    Again Again, command was understood but the write may have failed 
                R4  ; Check if the write was succeesfull
                    ; Returns 1     Success 
                EO=1    ; Driver reads/writes turn of the motor, this turns it back on
                    ; Returns OK


        "
    );
}

pub fn interactive_mode(handle: &DeviceHandle<GlobalContext>) -> Result<()> {
    writeln!(std::io::stdout(), "Entering interactive mode\n").unwrap();
    let (mut raw_command, mut command, mut response): (String, String, String);
    loop {
        raw_command = String::new();
        match stdin().read_line(&mut raw_command) {
            Ok(_n) => (),
            Err(e) => writeln!(std::io::stderr(), "Failed to read line with error {}", e).unwrap(),
        }

        command = raw_command.trim().to_ascii_uppercase();

        match command.as_str() {
            "EXIT" => break,
            "HELP" => _print_interactive_mode_help_message(),
            _ => {
                response =
                    send_command_get_response(handle, &[command.as_bytes(), b"\0"].concat())?;

                // Move to begining of previous line then clear line
                // Gives a psudo-way to delete the previous line
                print!("\x1b[1F\x1b[2K");

                println!("{} --> {}", command, response);
            }
        };
    }
    println!("Exiting interactive mode\n");
    Ok(())
}
