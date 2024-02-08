use crate::stage_control::commands::{
    get_high_speed, move_cycle_get_time, move_stage, set_acceleration_time, set_deceleration_time,
    set_encoder_position, set_high_speed, set_idle_time, set_low_speed, set_microstepping,
    set_movement_type, set_pulse_position, turn_motor_on, wait_for_motor_idle,
    write_driver_settings,
};

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{BufWriter, Read},
    thread::sleep,
    time::Instant,
};

use super::commands::{set_idle_current, set_run_current};

#[derive(Debug)]
pub struct RunParameters {
    high_speed: u32, // Starting high speed inputed by user, doesn't change
    low_speed: u32,
    acceleration_time: u32,
    deceleration_time: u32,
    idle_time: u32,
    amplitude: i32,
    period: f64,
    dwell_time: f64,
    factor: f64,
    time: Option<Instant>, // This is the current cycle time, changes every cycle
    hspd: u32, // This is the current high speed and not the inputted high speed, changes every cycle
    load_cycles: u32,
    offset: i32,
}

pub fn read_file_to_vector_of_lines(file_path: &str) -> std::io::Result<Vec<String>> {
    let mut file: File = File::open(file_path)?;
    let mut whole_file: String = String::new();
    file.read_to_string(&mut whole_file)?;
    let split_vec: Vec<String> = whole_file.split("\n").map(str::to_string).collect();
    return Ok(split_vec);
}

fn initialize_run_parameters() -> RunParameters {
    return RunParameters {
        high_speed: 0u32, // Starting high speed inputed by user, doesn't change
        low_speed: 0u32,
        acceleration_time: 0u32,
        deceleration_time: 0u32,
        idle_time: 0u32,
        amplitude: 0i32,
        period: 0f64,
        dwell_time: 0f64,
        time: None,
        factor: 2f64,
        hspd: 0u32, // This is the current high speed and not the inputted high speed, changes every cycle
        load_cycles: 0u32,
        offset: 0i32,
    };
}

fn adjust_speed(
    handle: &DeviceHandle<GlobalContext>,
    params: &RunParameters,
    time: f64,
    cycle: u32,
) -> rusb::Result<u32> {
    let error = (time - params.period * cycle as f64) * params.factor * 1000.0;
    let new_hspd: u32 = (params.hspd as i32 + error as i32) as u32;
    set_high_speed(handle, new_hspd)?;
    println!(
        "time={}\tperiod={}\tnewhspd={}\terror={}\thspd{})",
        time,
        params.period * cycle as f64,
        new_hspd,
        error,
        get_high_speed(handle)?,
    );
    return Ok(new_hspd);
}

fn set_run_parameters_from_file(
    handle: &DeviceHandle<GlobalContext>,
    file_path: &str,
) -> rusb::Result<RunParameters> {
    let mut params: RunParameters = initialize_run_parameters();

    let lines =
        read_file_to_vector_of_lines(file_path).expect("Couldn't read inputs to run device");

    for whole_line in lines {
        let line: Vec<&str> = whole_line.split_whitespace().collect();

        if whole_line.is_empty() || line[0].starts_with("#") {
            continue;
        }

        match line[0].to_ascii_lowercase().as_str() {
            "highspeed" => {
                let _ = set_high_speed(handle, line[1].parse::<u32>().unwrap())?;
                params.high_speed = line[1].parse().unwrap();
                params.hspd = line[1].parse().unwrap();
            }
            "lowspeed" => {
                let _ = set_low_speed(handle, line[1].parse::<u32>().unwrap())?;
                params.low_speed = line[1].parse().unwrap();
            }
            "accelerationtime" => {
                let _ = set_acceleration_time(handle, line[1].parse::<u32>().unwrap())?;
                params.acceleration_time = line[1].parse().unwrap();
            }
            "decelerationtime" => {
                let _ = set_deceleration_time(handle, line[1].parse::<u32>().unwrap())?;
                params.deceleration_time = line[1].parse().unwrap();
            }
            "idletime" => {
                let _ = set_idle_time(handle, line[1].parse::<u32>().unwrap())?;
                params.idle_time = line[1].parse().unwrap();
            }

            "amplitude" => params.amplitude = line[1].parse().unwrap(),
            "dwelltime" => params.dwell_time = line[1].parse().unwrap(),
            "factor" => params.factor = line[1].parse().unwrap(),
            "period" => params.period = line[1].parse().unwrap(),
            "offset" => params.offset = line[1].parse().unwrap(),
            "loadcycles" => params.load_cycles = line[1].parse().unwrap(),

            _ => println!(
                "Couldn't understand {:?}",
                line[0].to_ascii_lowercase().as_str()
            ),
        }
    }
    return Ok(params);
}

pub fn run_prep(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<RunParameters> {
    set_microstepping(handle, 50)?;
    set_idle_current(handle, 100)?;
    set_run_current(handle, 2000)?;
    set_movement_type(handle, "inc")?;
    set_pulse_position(handle, 0)?;
    set_encoder_position(handle, 0)?;
    let params = set_run_parameters_from_file(handle, "./input_output_files/RunInput.txt")?;
    write_driver_settings(handle)?;
    turn_motor_on(handle)?;
    return Ok(params);
}

pub fn run(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let mut params = run_prep(handle)?;
    let pos_file = &mut Some(BufWriter::new(
        File::create("./input_output_files/RunOutput.txt").unwrap(),
    ));

    set_high_speed(handle, 1500)?;
    move_stage(handle, -params.offset)?;
    set_high_speed(handle, params.hspd)?;
    wait_for_motor_idle(handle, &mut None, None)?;
    sleep(std::time::Duration::from_secs(1));

    let time = Some(Instant::now());
    for cycle in 1..params.load_cycles + 1 {
        move_cycle_get_time(handle, params.amplitude, pos_file, time, params.dwell_time)?;
        adjust_speed(
            handle,
            &params,
            time.unwrap().elapsed().as_secs_f64(),
            cycle,
        )?;
    }
    move_stage(handle, params.offset + 4913)?;
    wait_for_motor_idle(handle, &mut None, None)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    return Ok(());
}
