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
    time::Instant,
};

#[derive(Debug)]
pub struct RunParameters {
    high_speed: u32,
    low_speed: u32,
    acceleration_time: u32,
    deceleration_time: u32,
    idle_time: u32,
    pub load_standby_cyles: u32,
    load_cycles: u32,
    amplitude: i32,
    offset: i32,
    period: f64,
    pub standby_duration: u32,
    dwell_time: f64,
}

pub fn read_file_to_vector_of_lines(file_path: &str) -> std::io::Result<Vec<String>> {
    let mut file: File = File::open(file_path)?;
    let mut whole_file: String = String::new();
    file.read_to_string(&mut whole_file)?;
    let split_vec: Vec<String> = whole_file.split("\n").map(str::to_string).collect();
    return Ok(split_vec);
}

fn _initialize_run_parameters() -> RunParameters {
    return RunParameters {
        high_speed: 0u32,
        low_speed: 0u32,
        acceleration_time: 0u32,
        deceleration_time: 0u32,
        idle_time: 0u32,
        load_standby_cyles: 0u32,
        load_cycles: 0u32,
        amplitude: 0i32,
        offset: 0i32,
        period: 0f64,
        standby_duration: 0u32,
        dwell_time: 0f64,
    };
}

fn _get_average_of_vector(vec: &Vec<f64>) -> f64 {
    let mut sum: f64 = 0.0;
    for ele in vec {
        sum = sum + ele;
    }
    return sum / vec.len() as f64;
}

fn _adjust_speed(
    handle: &DeviceHandle<GlobalContext>,
    index: u32,
    factor: f64,
    period: f64,
    time: f64,
    hspd: u32,
) -> rusb::Result<u32> {
    let error = time - period * (index as f64);
    let new_hspd = hspd as i32 + ((error * factor * 1000.0) as i32);
    set_high_speed(handle, new_hspd as u32)?;
    return Ok(new_hspd as u32);
}

pub fn set_run_parameters_from_file(
    handle: &DeviceHandle<GlobalContext>,
    file_path: &str,
) -> rusb::Result<RunParameters> {
    let mut params: RunParameters = _initialize_run_parameters();
    let lines =
        read_file_to_vector_of_lines(file_path).expect("Couldn't read inputs to run device");
    for whole_line in lines {
        let line: Vec<&str> = whole_line.split_whitespace().collect();
        if whole_line.is_empty() || line[0].starts_with("#") {
            continue;
        }
        match line[0].to_ascii_lowercase().as_str() {
            "highspeed" => {
                set_high_speed(handle, line[1].parse::<u32>().unwrap())?;
                params.high_speed = line[1].parse().unwrap();
            }
            "lowspeed" => {
                set_low_speed(handle, line[1].parse::<u32>().unwrap())?;
                params.low_speed = line[1].parse().unwrap();
            }
            "accelerationtime" => {
                set_acceleration_time(handle, line[1].parse::<u32>().unwrap())?;
                params.acceleration_time = line[1].parse().unwrap();
            }
            "decelerationtime" => {
                set_deceleration_time(handle, line[1].parse::<u32>().unwrap())?;
                params.deceleration_time = line[1].parse().unwrap();
            }
            "idletime" => {
                set_idle_time(handle, line[1].parse::<u32>().unwrap())?;
                params.idle_time = line[1].parse().unwrap();
            }
            "loadstandbycyles" => params.load_standby_cyles = line[1].parse().unwrap(),
            "amplitude" => params.amplitude = line[1].parse().unwrap(),
            "offset" => params.offset = line[1].parse().unwrap(),
            "period" => params.period = line[1].parse().unwrap(),
            "loadcycles" => params.load_cycles = line[1].parse().unwrap(),
            "standbyduration" => params.standby_duration = line[1].parse().unwrap(),
            "dwelltime" => params.dwell_time = line[1].parse().unwrap(),
            _ => println!(
                "Couldn't understand {:?}",
                line[0].to_ascii_lowercase().as_str()
            ),
        }
    }
    write_driver_settings(handle)?;
    turn_motor_on(handle)?;
    return Ok(params);
}

pub fn run_prep(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<RunParameters> {
    set_microstepping(handle, 50)?;
    set_movement_type(handle, "inc")?;
    set_pulse_position(handle, 0)?;
    set_encoder_position(handle, 0)?;
    let params = set_run_parameters_from_file(handle, "./RunInput.txt")?;
    return Ok(params);
}

pub fn run(handle: &DeviceHandle<GlobalContext>, params: &RunParameters) -> rusb::Result<()> {
    let start_hspd: u32 = get_high_speed(handle)?;
    let pos_file = &mut Some(BufWriter::new(File::create("RunOutput.txt").unwrap()));
    let total_time = Some(Instant::now());
    for load_cycle in 1..params.load_cycles + 1 {
        move_cycle_get_time(
            handle,
            params.amplitude,
            pos_file,
            total_time,
            params.dwell_time,
        )?;
        _adjust_speed(
            handle,
            load_cycle,
            2.0,
            params.period,
            total_time.unwrap().elapsed().as_secs_f64(),
            start_hspd,
        )?;
    }
    move_stage(handle, params.offset + 4913)?;
    wait_for_motor_idle(handle, &mut None, None)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    return Ok(());
}
