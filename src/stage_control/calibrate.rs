use crate::stage_control::control::{
    get_cycle_time, get_high_speed, set_acceleration_time, set_deceleration_time,
    set_encoder_position, set_high_speed, set_idle_time, set_low_speed, set_microstepping,
    set_movement_type, set_pulse_position, turn_motor_on, write_driver_settings,
};

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{Read, Write},
    time::Instant,
};

#[derive(Debug)]
pub struct CalibrateParameters {
    high_speed: u64,
    low_speed: u64,
    acceleration_time: u64,
    deceleration_time: u64,
    idle_time: u64,
    averaging_cycles: u64,
    amplitude: i64,
    period: f64,
    dwell_time: f64,
    factor: f64,
    max_speed: u64,
    tolerance: f64,
}

pub fn read_file_to_vector_of_lines(file_path: &str) -> std::io::Result<Vec<String>> {
    let mut file: File = File::open(file_path)?;

    let mut whole_file: String = String::new();

    file.read_to_string(&mut whole_file)?;

    let split_vec: Vec<String> = whole_file.split("\n").map(str::to_string).collect();

    return Ok(split_vec);
}

fn _initialize_calibrate_parameters() -> CalibrateParameters {
    return CalibrateParameters {
        high_speed: 0u64,
        low_speed: 0u64,
        acceleration_time: 0u64,
        deceleration_time: 0u64,
        idle_time: 0u64,
        averaging_cycles: 0u64,
        amplitude: 0i64,
        period: 0f64,
        dwell_time: 0f64,
        factor: 0f64,
        max_speed: 0u64,
        tolerance: 0f64,
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
    handle: DeviceHandle<GlobalContext>,
    index: u64,
    factor: f64,
    period: f64,
    time: f64,
    hspd: u64,
) -> rusb::Result<u64> {
    let error = time - period * (index as f64);

    let new_hspd = hspd as i64 + ((error * factor * 1000.0) as i64);

    match set_high_speed(handle, new_hspd as u64) {
        Ok(()) => return Ok(new_hspd as u64),
        Err(e) => return Err(e),
    };
}

pub fn set_calibrate_parameters_from_file(
    handle: DeviceHandle<GlobalContext>,
    file_path: &str,
) -> rusb::Result<CalibrateParameters> {
    let mut params: CalibrateParameters = _initialize_calibrate_parameters();

    let lines =
        read_file_to_vector_of_lines(file_path).expect("Couldn't read inputs to run device");

    for whole_line in lines {
        let line: Vec<&str> = whole_line.split_whitespace().collect();

        if whole_line.is_empty() || line[0].starts_with("#") {
            continue;
        }

        match line[0].to_ascii_lowercase().as_str() {
            "highspeed" => {
                match set_high_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
                params.high_speed = line[1].parse().unwrap();
            }
            "lowspeed" => {
                match set_low_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
                params.low_speed = line[1].parse().unwrap();
            }
            "accelerationtime" => {
                match set_acceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
                params.acceleration_time = line[1].parse().unwrap();
            }
            "decelerationtime" => {
                match set_deceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
                params.deceleration_time = line[1].parse().unwrap();
            }
            "idletime" => {
                match set_idle_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => return Err(e),
                };
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

    match write_driver_settings(handle) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };
    match turn_motor_on(handle) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    return Ok(params);
}

pub fn calibrate(handle: DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let params = match set_calibrate_parameters_from_file(handle, "./CalibrateInput.txt") {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    let min_period: f64 = params.period * params.tolerance;
    let max_period: f64 = params.period * (2.0 - params.tolerance);

    let mut hspd: u64 = match get_high_speed(handle) {
        Ok(hspd) => hspd,
        Err(e) => return Err(e),
    };
    let mut time: f64 = 0.0;
    let mut times: Vec<f64> = vec![0.0; params.averaging_cycles as usize];

    let pos_file = &mut File::create("./CalibrateOutput.txt").unwrap();

    match set_microstepping(handle, 50) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };
    match set_movement_type(handle, "inc") {
        Ok(()) => (),
        Err(e) => return Err(e),
    };
    match set_pulse_position(handle, 0) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };
    match set_encoder_position(handle, 0) {
        Ok(()) => (),
        Err(e) => return Err(e),
    };

    while time < min_period || time > max_period {
        let total_time = Instant::now();
        for i in 0..params.averaging_cycles as usize {
            times[i] = get_cycle_time(
                handle,
                params.amplitude,
                pos_file,
                total_time,
                params.dwell_time,
            )?;
        }

        time = _get_average_of_vector(&times);

        hspd = _adjust_speed(handle, 1, params.factor, params.period, time, hspd)?;

        println!("{}\n{}\n", time, hspd);

        if hspd > params.max_speed {
            println!("Max high speed tripped! Value was {}.", hspd);
            return Err(rusb::Error::Overflow);
        }
    }

    println!("Calibration complete. Parameters outputted to 'RunInput_calibrated'");

    let mut out_file = File::create("RunInput_calibrated").unwrap();
    out_file
        .write_all(
            &[
                b"HighSpeed ",
                hspd.to_string().as_bytes(),
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
                b"LoadStandbyCyles ",
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
                b"StandbyDuration ",
                b"\n",
                b"DwellTime ",
                params.dwell_time.to_string().as_bytes(),
                b"\n\n\n",
            ]
            .concat(),
        )
        .unwrap();

    out_file
        .write_all(
            &[
                b"# Calibration performed with: \n# Averaging cyles: ",
                params.averaging_cycles.to_string().as_bytes(),
                b"\n",
                b"# Final period: ",
                time.to_string().as_bytes(),
            ]
            .concat(),
        )
        .unwrap();
    return Ok(());
}
