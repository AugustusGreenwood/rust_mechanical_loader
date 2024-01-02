use crate::stage_control::driver::send_command_get_response;

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{stdin, Write},
    time::{Duration, Instant},
};

pub fn write_driver_settings(handle: DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match send_command_get_response(handle, "RW\0".as_bytes()) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    std::thread::sleep(std::time::Duration::from_secs(3));

    match send_command_get_response(handle, "R4\0".as_bytes()) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    }
    return Ok(());
}

pub fn set_high_speed(
    handle: DeviceHandle<GlobalContext>,
    new_high_speed: u64,
) -> rusb::Result<()> {
    let command = &[
        "HSPD=".as_bytes(),
        &new_high_speed.to_string().as_bytes(),
        &[0],
    ]
    .concat();
    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_low_speed(handle: DeviceHandle<GlobalContext>, new_low_speed: u64) -> rusb::Result<()> {
    let command = &[
        "LSPD=".as_bytes(),
        &new_low_speed.to_string().as_bytes(),
        &[0],
    ]
    .concat();
    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_acceleration_time(handle: DeviceHandle<GlobalContext>, time: u64) -> rusb::Result<()> {
    let command = &["ACC=".as_bytes(), &time.to_string().as_bytes(), &[0]].concat();
    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_acceleration_profile(
    handle: DeviceHandle<GlobalContext>,
    sin_trap: &str,
) -> rusb::Result<()> {
    let sin_com = &["SCV=1".as_bytes(), "\0".as_bytes()].concat();
    let trap_com = &["SCV=0".as_bytes(), "\0".as_bytes()].concat();
    let command = match sin_trap {
        "sin" => sin_com,
        "trap" => trap_com,
        _ => return Err(rusb::Error::Other),
    };

    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_deceleration_time(handle: DeviceHandle<GlobalContext>, time: u64) -> rusb::Result<()> {
    let command = &["DEC=".as_bytes(), &time.to_string().as_bytes(), &[0]].concat();
    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_idle_time(handle: DeviceHandle<GlobalContext>, time: u64) -> rusb::Result<()> {
    if time < 1 || time > 100 {
        return Err(rusb::Error::InvalidParam);
    }

    match send_command_get_response(
        handle,
        &[
            "DRVIT=".as_bytes(),
            &time.to_string().as_bytes(),
            "\0".as_bytes(),
        ]
        .concat(),
    ) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn turn_motor_on(handle: DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    match send_command_get_response(handle, "EO=1\0".as_bytes()) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    }
    std::thread::sleep(Duration::from_secs(3));
    return Ok(());
}

pub fn set_microstepping(handle: DeviceHandle<GlobalContext>, microsteps: u64) -> rusb::Result<()> {
    if microsteps < 2 || microsteps > 500 {
        return Err(rusb::Error::InvalidParam);
    }

    match send_command_get_response(
        handle,
        &[
            "DRVMS=".as_bytes(),
            &microsteps.to_string().as_bytes(),
            "\0".as_bytes(),
        ]
        .concat(),
    ) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_movement_type(handle: DeviceHandle<GlobalContext>, abs_inc: &str) -> rusb::Result<()> {
    let abs_com = &["ABS".as_bytes(), "\0".as_bytes()].concat();
    let inc_com = &["INC".as_bytes(), "\0".as_bytes()].concat();
    let command = match abs_inc.to_ascii_lowercase().as_str() {
        "abs" => abs_com,
        "inc" => inc_com,
        _ => return Err(rusb::Error::InvalidParam),
    };

    match send_command_get_response(handle, command) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn get_high_speed(handle: DeviceHandle<GlobalContext>) -> rusb::Result<u64> {
    let response: u64 = match send_command_get_response(handle, "HSPD\0".as_bytes()) {
        Ok(response) => response.parse::<u64>().unwrap(),
        Err(e) => return Err(e),
    };
    return Ok(response);
}

pub fn set_pulse_position(handle: DeviceHandle<GlobalContext>, position: i64) -> rusb::Result<()> {
    match send_command_get_response(
        handle,
        &[
            "PX=".as_bytes(),
            &position.to_string().as_bytes(),
            "\0".as_bytes(),
        ]
        .concat(),
    ) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn set_encoder_position(
    handle: DeviceHandle<GlobalContext>,
    position: i64,
) -> rusb::Result<()> {
    match send_command_get_response(
        handle,
        &[
            "EX=".as_bytes(),
            &position.to_string().as_bytes(),
            "\0".as_bytes(),
        ]
        .concat(),
    ) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn get_pulse_position(handle: DeviceHandle<GlobalContext>) -> rusb::Result<i64> {
    let response =
        match send_command_get_response(handle, &["PX".as_bytes(), "\0".as_bytes()].concat()) {
            Ok(response) => response.parse::<i64>().unwrap(),
            Err(e) => return Err(e),
        };
    return Ok(response);
}

pub fn get_encoder_position(handle: DeviceHandle<GlobalContext>) -> rusb::Result<i64> {
    let response =
        match send_command_get_response(handle, &["EX".as_bytes(), "\0".as_bytes()].concat()) {
            Ok(response) => response.parse::<i64>().unwrap(),
            Err(e) => return Err(e),
        };
    return Ok(response);
}

pub fn move_stage(handle: DeviceHandle<GlobalContext>, position: i64) -> rusb::Result<()> {
    match send_command_get_response(
        handle,
        &[
            "X".as_bytes(),
            &position.to_string().as_bytes(),
            "\0".as_bytes(),
        ]
        .concat(),
    ) {
        Ok(_response) => (),
        Err(e) => return Err(e),
    };
    return Ok(());
}

pub fn get_motor_status(handle: DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = match send_command_get_response(handle, "MST\0".as_bytes()) {
        Ok(response) => response.parse::<i32>().unwrap(),
        Err(e) => return Err(e),
    };
    return Ok(response);
}

pub fn output_time_pos_to_file(
    handle: DeviceHandle<GlobalContext>,
    file: &mut File,
    time: Instant,
) -> rusb::Result<()> {
    file.write_all(
        &[
            time.elapsed().as_secs_f64().to_string().as_bytes(),
            "\t".as_bytes(),
            get_pulse_position(handle)?.to_string().as_bytes(),
            "\t".as_bytes(),
            get_encoder_position(handle)?.to_string().as_bytes(),
            "\n".as_bytes(),
        ]
        .concat(),
    )
    .unwrap();
    return Ok(());
}

pub fn wait_for_motor_idle(
    handle: DeviceHandle<GlobalContext>,
    file: Option<&mut File>,
    time: Option<Instant>,
) -> rusb::Result<()> {
    match file {
        None => while get_motor_status(handle)? != 0 {},
        Some(file) => {
            while get_motor_status(handle)? != 0 {
                output_time_pos_to_file(handle, file, time.unwrap())?;
            }
        }
    }
    return Ok(());
}

pub fn get_cycle_time(
    handle: DeviceHandle<GlobalContext>,
    distance: i64,
    file: &mut File,
    total_time: Instant,
    dwell: f64,
) -> rusb::Result<f64> {
    let time: Instant = Instant::now();
    output_time_pos_to_file(handle, file, total_time)?;
    move_stage(handle, -distance)?;
    wait_for_motor_idle(handle, Some(file), Some(total_time))?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell - 0.005));
    output_time_pos_to_file(handle, file, total_time)?;
    move_stage(handle, distance)?;
    wait_for_motor_idle(handle, Some(file), Some(total_time))?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell - 0.005));
    return Ok(time.elapsed().as_secs_f64());
}

pub fn interactive_mode(handle: DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    writeln!(std::io::stdout(), "Entering interactive mode").unwrap();
    let (mut raw_command, mut command, mut response): (String, String, String);
    loop {
        raw_command = String::new();
        match stdin().read_line(&mut raw_command) {
            Ok(_n) => (),
            Err(e) => writeln!(std::io::stderr(), "Failed to read line with error {}", e).unwrap(),
        }

        command = raw_command.trim().to_ascii_uppercase();

        if command == "EXIT" {
            break;
        }

        print!("\x1b[1F\x1b[2K"); // Move to begining of previous line then clear line

        response = match send_command_get_response(
            handle,
            &[command.as_bytes(), "\0".as_bytes()].concat(),
        ) {
            Ok(r) => r,
            Err(_e) => "Command not understood!".to_string(),
        };

        println!("{} --> {}", command, response);
    }
    println!("Exiting interactive mode");
    return Ok(());
}
