use crate::stage_control::commands::{
    get_high_speed, move_cycle_get_time, set_acceleration_time, set_deceleration_time,
    set_encoder_position, set_high_speed, set_idle_time, set_low_speed, set_microstepping,
    set_movement_type, set_pulse_position, turn_motor_on, write_driver_settings,
};

use rusb::{DeviceHandle, GlobalContext};

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    time::Instant,
};

#[derive(Debug)]
pub struct CalibrateParameters {
    high_speed: u32,
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
        high_speed: 0u32,
        low_speed: 0u32,
        acceleration_time: 0u32,
        deceleration_time: 0u32,
        idle_time: 0u32,
        averaging_cycles: 0u32,
        amplitude: 0i32,
        period: 0f64,
        dwell_time: 0f64,
        factor: 0f64,
        max_speed: 0u32,
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
    handle: &DeviceHandle<GlobalContext>,
    index: u32,
    factor: f64,
    period: f64,
    time: f64,
    hspd: u32,
) -> rusb::Result<u32> {
    let error = time - period * (index as f64);
    let new_hspd: u32 = (hspd as i32 + (error * factor * 1000.0) as i32) as u32;
    set_high_speed(handle, new_hspd)?;
    return Ok(new_hspd);
}

pub fn set_calibrate_parameters_from_file(
    handle: &DeviceHandle<GlobalContext>,
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
                let _ = set_high_speed(handle, line[1].parse::<u32>().unwrap())?;
                params.high_speed = line[1].parse().unwrap();
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

    let _ = write_driver_settings(handle)?;
    let _ = turn_motor_on(handle)?;

    return Ok(params);
}

pub fn calibrate(handle: &DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let params = set_calibrate_parameters_from_file(handle, "./CalibrateInput.txt")?;

    let min_period: f64 = params.period * params.tolerance;
    let max_period: f64 = params.period * (2.0 - params.tolerance);

    let mut hspd: u32 = get_high_speed(handle)?;
    let mut time: f64 = 0.0;
    let mut times: Vec<f64> = vec![0.0; params.averaging_cycles as usize];

    let pos_file = &mut Some(BufWriter::new(
        File::create("./CalibrateOutput.txt").unwrap(),
    ));

    let _ = set_microstepping(handle, 50)?;
    let _ = set_movement_type(handle, "inc")?;
    let _ = set_pulse_position(handle, 0)?;
    let _ = set_encoder_position(handle, 0)?;

    while time < min_period || time > max_period {
        let total_time = Some(Instant::now());
        for i in 0..params.averaging_cycles as usize {
            times[i] = move_cycle_get_time(
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
