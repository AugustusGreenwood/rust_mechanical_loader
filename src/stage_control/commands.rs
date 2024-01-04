use crate::stage_control::driver::{
    get_handle_from_vendor_product_id, read_from_bulk, saftey_read, write_to_bulk, write_to_control,
};

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{stdin, BufWriter, Write},
    time::{Duration, Instant},
};

pub fn open(vendor_id: u16, product_id: u16) -> rusb::Result<DeviceHandle<GlobalContext>> {
    let mut handle: DeviceHandle<GlobalContext> =
        get_handle_from_vendor_product_id(vendor_id, product_id)?;

    handle.claim_interface(0)?;
    write_to_control(&handle, 2)?;
    saftey_read(&handle)?;
    return Ok(handle);
}

pub fn close(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    write_to_control(handle, 4)?;
    handle.release_interface(0)?;
    return Ok(());
}

pub fn send_command_get_response(
    handle: &DeviceHandle<GlobalContext>,
    command: &[u8],
) -> rusb::Result<String> {
    saftey_read(handle)?;
    write_to_bulk(handle, command)?;
    return read_from_bulk(handle);
}

pub fn write_driver_settings(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let _ = send_command_get_response(handle, b"RW\0")?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    let _ = send_command_get_response(handle, b"R4\0")?;
    return Ok(());
}

pub fn set_high_speed(
    handle: &DeviceHandle<GlobalContext>,
    new_high_speed: u32,
) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"HSPD=", new_high_speed.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn move_stage(handle: &DeviceHandle<GlobalContext>, position: i32) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"X", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_low_speed(handle: &DeviceHandle<GlobalContext>, new_low_speed: u32) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"HSPD=", new_low_speed.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_acceleration_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"ACC=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_acceleration_profile(
    handle: &DeviceHandle<GlobalContext>,
    sin_trap: &str,
) -> rusb::Result<()> {
    let command = match sin_trap {
        "sin" => b"SCV=1\0",
        "trap" => b"SCV=0\0",
        _ => return Err(rusb::Error::Other),
    };

    let _ = send_command_get_response(handle, command)?;
    return Ok(());
}

pub fn set_deceleration_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"DEC=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_idle_time(handle: &DeviceHandle<GlobalContext>, time: u32) -> rusb::Result<()> {
    if time < 1 || time > 100 {
        return Err(rusb::Error::InvalidParam);
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVIT=", time.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn turn_motor_on(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let _ = send_command_get_response(handle, b"EO=1\0")?;
    std::thread::sleep(Duration::from_secs(3));
    return Ok(());
}

pub fn set_microstepping(
    handle: &DeviceHandle<GlobalContext>,
    microsteps: u32,
) -> rusb::Result<()> {
    if microsteps < 2 || microsteps > 500 {
        return Err(rusb::Error::InvalidParam);
    }

    let _ = send_command_get_response(
        handle,
        &[b"DRVMS=", microsteps.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_movement_type(handle: &DeviceHandle<GlobalContext>, abs_inc: &str) -> rusb::Result<()> {
    let command = match abs_inc.to_ascii_lowercase().as_str() {
        "abs" => b"ABS\0",
        "inc" => b"INC\0",
        _ => return Err(rusb::Error::InvalidParam),
    };

    let _ = send_command_get_response(handle, command)?;
    return Ok(());
}

pub fn get_high_speed(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<u32> {
    let response: u32 = send_command_get_response(handle, b"HSPD\0")?
        .parse()
        .unwrap();
    return Ok(response);
}

pub fn set_pulse_position(handle: &DeviceHandle<GlobalContext>, position: i32) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"PX=", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn set_encoder_position(
    handle: &DeviceHandle<GlobalContext>,
    position: i32,
) -> rusb::Result<()> {
    let _ = send_command_get_response(
        handle,
        &[b"EX=", position.to_string().as_bytes(), b"\0"].concat(),
    )?;
    return Ok(());
}

pub fn get_pulse_position(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = send_command_get_response(handle, b"PX\0")?.parse().unwrap();
    return Ok(response);
}

pub fn get_encoder_position(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = send_command_get_response(handle, b"EX\0")?.parse().unwrap();
    return Ok(response);
}

pub fn get_motor_status(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<i32> {
    let response: i32 = send_command_get_response(handle, b"MST\0")?
        .parse()
        .unwrap();
    return Ok(response);
}

pub fn output_time_pos_to_file(
    handle: &DeviceHandle<GlobalContext>,
    file: &mut BufWriter<File>,
    time: Instant,
) -> rusb::Result<()> {
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
    return Ok(());
}

pub fn wait_for_motor_idle(
    handle: &DeviceHandle<GlobalContext>,
    file: &mut Option<BufWriter<File>>,
    time: Option<Instant>,
) -> rusb::Result<()> {
    match file {
        None => while get_motor_status(handle)? != 0 {},
        Some(file) => {
            while get_motor_status(handle)? != 0 {
                output_time_pos_to_file(handle, file, time.unwrap())?;
            }
            file.flush().unwrap();
        }
    }
    return Ok(());
}

pub fn move_cycle_get_time(
    handle: &DeviceHandle<GlobalContext>,
    distance: i32,
    file: &mut Option<BufWriter<File>>,
    time: Option<Instant>,
    dwell: f64,
) -> rusb::Result<f64> {
    let cycle_time = Instant::now();
    move_stage(handle, -distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(Duration::from_secs_f64(dwell));
    move_stage(handle, distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(Duration::from_secs_f64(dwell));
    return Ok(cycle_time.elapsed().as_secs_f64());
}

pub fn move_cycle(
    handle: &DeviceHandle<GlobalContext>,
    distance: i32,
    file: &mut Option<BufWriter<File>>,
    time: Option<Instant>,
    dwell: f64,
) -> rusb::Result<()> {
    move_stage(handle, -distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell));
    move_stage(handle, distance)?;
    wait_for_motor_idle(handle, file, time)?;
    std::thread::sleep(std::time::Duration::from_secs_f64(dwell));
    return Ok(());
}

pub fn interactive_mode(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
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

        response = send_command_get_response(handle, &[command.as_bytes(), b"\0"].concat())?;

        println!("{} --> {}", command, response);
    }
    println!("Exiting interactive mode");
    return Ok(());
}
