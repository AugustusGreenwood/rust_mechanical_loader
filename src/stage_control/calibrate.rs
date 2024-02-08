use crate::stage_control::commands::{
    move_cycle_get_time, set_acceleration_time, set_deceleration_time, set_encoder_position,
    set_high_speed, set_idle_time, set_low_speed, set_microstepping, set_movement_type,
    set_pulse_position, turn_motor_on, write_driver_settings,
};

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{Read, Write},
};

// TODO fix this? I don't like it. Better names, maybe different structure entirely
#[derive(Debug)]
pub struct CalibrateParameters {
    high_speed: u32, // Starting high speed inputed by user, doesn't change
    low_speed: u32,
    acceleration_time: u32,
    deceleration_time: u32,
    idle_time: u32,
    averaging_cycles: u32,
    amplitude: i32,
    period: f64,
    dwell_time: f64,
    factor: f64,
    max_speed: u32,
    tolerance: f64,
    min_period: f64,
    max_period: f64,
    time: f64, // This is the current cycle time, changes every cycle
    hspd: u32, // This is the current high speed and not the inputted high max_speed, changes every cycle
}

fn read_file_to_vector_of_lines(file_path: &str) -> std::io::Result<Vec<String>> {
    let mut file: File = File::open(file_path)?;
    let mut whole_file: String = String::new();
    file.read_to_string(&mut whole_file)?;
    let split_vec: Vec<String> = whole_file.split("\n").map(str::to_string).collect();
    return Ok(split_vec);
}

// TODO make thise bad numbers, I'd rather error then let a user use defaults
fn initialize_calibrate_parameters() -> CalibrateParameters {
    return CalibrateParameters {
        high_speed: 1500u32,
        low_speed: 100u32,
        acceleration_time: 1u32,
        deceleration_time: 1u32,
        idle_time: 1u32,
        averaging_cycles: 3u32,
        amplitude: 100i32,
        period: 3f64,
        dwell_time: 0.5f64,
        factor: 1f64,
        max_speed: 1000000u32,
        tolerance: 0.997f64,
        min_period: 0f64,
        max_period: 0f64,
        time: 0f64,
        hspd: 0u32,
    };
}

fn get_average_of_vector(vec: &Vec<f64>) -> f64 {
    let mut sum: f64 = 0.0;
    for ele in vec {
        sum = sum + ele;
    }
    return sum / vec.len() as f64;
}

fn adjust_speed(
    handle: &DeviceHandle<GlobalContext>,
    params: &CalibrateParameters,
) -> rusb::Result<u32> {
    let error = (params.time - params.period) * params.factor * 1000.0;
    let new_hspd: u32 = (params.hspd as i32 + error as i32) as u32;
    set_high_speed(handle, new_hspd)?;
    return Ok(new_hspd);
}

fn set_calibrate_parameters_from_file(
    handle: &DeviceHandle<GlobalContext>,
    file_path: &str,
) -> rusb::Result<CalibrateParameters> {
    let mut params: CalibrateParameters = initialize_calibrate_parameters();

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
            "averagingcycles" => params.averaging_cycles = line[1].parse().unwrap(),
            "dwelltime" => params.dwell_time = line[1].parse().unwrap(),
            "factor" => params.factor = line[1].parse().unwrap(),
            "maxspeed" => params.max_speed = line[1].parse().unwrap(),
            "period" => params.period = line[1].parse().unwrap(),
            "tolerance" => params.tolerance = line[1].parse().unwrap(),

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

fn write_run_file_after_calibration(params: CalibrateParameters) -> std::io::Result<()> {
    let mut out_file = File::create("./input_output_files/RunInput_calibrated.txt")?;
    out_file.write_all(
        &[
            b"HighSpeed ",
            params.hspd.to_string().as_bytes(),
            b"\n",
            b"LowSpeed ",
            params.low_speed.to_string().as_bytes(),
            b"\n",
            b"AccelerationTime ",
            params.acceleration_time.to_string().as_bytes(),
            b"\n",
            b"DecelerationTime ",
            params.deceleration_time.to_string().as_bytes(),
            b"\n",
            b"IdleTime ",
            params.idle_time.to_string().as_bytes(),
            b"\n",
            b"LoadCycles ",
            b"\n",
            b"Amplitude ",
            params.amplitude.to_string().as_bytes(),
            b"\n",
            b"Offset ",
            b"\n",
            b"Period ",
            params.period.to_string().as_bytes(),
            b"\n",
            b"DwellTime ",
            params.dwell_time.to_string().as_bytes(),
            b"\n\n\n",
        ]
        .concat(),
    )?;

    out_file.write_all(
        &[
            b"# Calibration performed with: \n# Averaging cyles: ",
            params.averaging_cycles.to_string().as_bytes(),
            b"\n",
            b"# Final period: ",
            params.time.to_string().as_bytes(),
        ]
        .concat(),
    )?;
    return Ok(());
}

fn prepare_for_calibration(
    handle: &DeviceHandle<GlobalContext>,
) -> rusb::Result<CalibrateParameters> {
    let params =
        set_calibrate_parameters_from_file(handle, "./input_output_files/CalibrateInput.txt")?;
    set_microstepping(handle, 50)?;
    set_movement_type(handle, "inc")?;
    set_pulse_position(handle, 0)?;
    set_encoder_position(handle, 0)?;
    return Ok(params);
}

fn calibration_loop(
    handle: &DeviceHandle<GlobalContext>,
    params: &mut CalibrateParameters,
) -> rusb::Result<()> {
    let mut times: Vec<f64> = vec![0.0; params.averaging_cycles as usize];

    while params.time < params.min_period || params.time > params.max_period {
        for i in 0..params.averaging_cycles as usize {
            times[i] =
                move_cycle_get_time(handle, params.amplitude, &mut None, None, params.dwell_time)?;
        }

        params.time = get_average_of_vector(&times);
        params.hspd = adjust_speed(handle, params)?;

        println!("{}\n{}\n", params.time, params.hspd);

        // This is really here to make sure the machine doesn't go crazy
        // I'm not really worried if the speed get too low, the user
        // can stop that
        if params.hspd > params.max_speed {
            println!("Max high speed tripped! Value was {}.", params.hspd);
            return Err(rusb::Error::Overflow); // Probably a better error for this
        }
    }
    return Ok(());
}

pub fn calibrate(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let mut params = prepare_for_calibration(handle)?; // Initialize params and set some important
                                                       // motor controls
    params.min_period = params.period * params.tolerance;
    params.max_period = params.period * (2.0 - params.tolerance);

    calibration_loop(handle, &mut params)?;

    println!("Calibration complete. Parameters outputted to 'RunInput_calibrated.txt'");

    write_run_file_after_calibration(params).unwrap();
    return Ok(());
}
