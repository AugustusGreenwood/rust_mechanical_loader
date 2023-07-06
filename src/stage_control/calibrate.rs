/* 
    File defines the useful function for calibrating the device. Some crossover with "run.rs"
*/

use crate::stage_control::control::{
    set_high_speed,
    set_low_speed,
    set_acceleration_time,
    set_deceleration_time,
    set_idle_time,
    set_movement_type,
    set_pulse_position,
    set_encoder_position,
    get_high_speed,
    write_driver_settings,
    turn_motor_on,
    get_cycle_time, set_microstepping,
};

use rusb::{
    DeviceHandle,
    GlobalContext
};

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
    tolerance: f64
}


// Because the files are quite small, I decided to load the whole file to memory and then 
// perform operations on that string when getting the inputs from file. This seemed 
// more succinct and easier than useing a buffer.
pub fn read_file_to_vector_of_lines(file_path: &str) -> std::io::Result<Vec<String>> {
    let mut file: File = File::open(file_path)?;
    
    let mut whole_file: String = String::new();
    
    file.read_to_string(&mut whole_file)?;

    let split_vec: Vec<String> = whole_file.split("\n").map(str::to_string).collect();

    return Ok(split_vec);
}


// This could be improved (maybe?) by using reasonably standard inputs? 
// This is really here to clean up the "set_calibrate_params_from_file" func
fn _initialize_calibrate_parameters() -> CalibrateParameters {
    return CalibrateParameters{high_speed: 0u64,
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
                            tolerance: 0f64};
}

fn _get_average_of_vector(vec: &Vec<f64>) -> f64 {
    let mut sum: f64 = 0.0;
    for ele in vec {
        sum = sum + ele;
    }

    return sum / vec.len() as f64;
}



// The difference between desired and actual time have a nice analog
// to the high speed when multiplied by 1000, BUT this isn't perfect. Sometimes it can move to fast or too slow depending on where you
// might be in finding the speed, so the 'factor' prarameter allows the user to dictate how big the jumps are. So a smaller factor 
// will have a slow calibration and may favor the low or high end of tolerance depending on where you start. A high factor will possibly have
// a faster calibration speed, but could just cycle around, or it could jump to the center of your tolerance range.
// The new high speed is returned because in the 'calibrate' function, the high speed set needs to outted to make sure the max high speed
// isn't tripped. It also is required because the new speed needs to be inputed back into the function on the next cycle.
fn _adjust_speed(handle: &mut DeviceHandle<GlobalContext>, index: u64, factor: f64, period: f64, time: f64, hspd: u64) -> rusb::Result<u64> {
    let error = time - period * (index as f64);

    let new_hspd = hspd as i64 + ((error * factor * 1000.0) as i64);

    match set_high_speed(handle, new_hspd as u64) {
        Ok(()) => return Ok(new_hspd as u64),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_high_speed' failed with error {} in 'stage_control::calibrate::_adjust_speed'", e).unwrap();
            return Err(e)
        },
    };
}


// Not very robust, but it does the job. Could be made better, but I don't know how. 
pub fn set_calibrate_parameters_from_file(handle: &mut DeviceHandle<GlobalContext>, file_path: &str) -> rusb::Result<CalibrateParameters> {
    let mut params: CalibrateParameters = _initialize_calibrate_parameters();
    
    let lines = read_file_to_vector_of_lines(file_path).expect("Couldn't read inputs to run device");
    
    for whole_line in lines {
        let line: Vec<&str> = whole_line.split_whitespace().collect();
        
        if whole_line.is_empty() || line[0].starts_with("#") {continue;}

        match line[0].to_ascii_lowercase().as_str() {
            "highspeed" => {
                match set_high_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_high_speed' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };
                params.high_speed = line[1].parse().unwrap();
            },
            "lowspeed" => {
                match set_low_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_low_speed' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };                
                params.low_speed = line[1].parse().unwrap();
            },
            "accelerationtime" => {
                match set_acceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_acceleration_time' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };                    
                params.acceleration_time = line[1].parse().unwrap();
            },
            "decelerationtime" => {
                match set_deceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_deceleration_time' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };  
                params.deceleration_time = line[1].parse().unwrap();
            },
            "idletime" => {
                match set_idle_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_idle_time' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };  
                params.idle_time = line[1].parse().unwrap();
            },

            "amplitude" => params.amplitude = line[1].parse().unwrap(),
            "averagingcycles" => params.averaging_cycles = line[1].parse().unwrap(),
            "dwelltime" => params.dwell_time = line[1].parse().unwrap(),
            "factor" => params.factor = line[1].parse().unwrap(),
            "maxspeed" => params.max_speed = line[1].parse().unwrap(),
            "period" => params.period = line[1].parse().unwrap(),
            "tolerance" => params.tolerance = line[1].parse().unwrap(),

            _ => println!("Couldn't understand {:?}", line[0].to_ascii_lowercase().as_str()),
        }
    }
    
    match write_driver_settings(handle) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::write_driver_settings' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap(),

    };
    match turn_motor_on(handle) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::turn_motor_on' failed with error {} in 'stage_control::calibrate::set_calibrate_parameteres_from_file'", e).unwrap(),

    };    
    
    return Ok(params);
}



pub fn calibrate(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<()> {
    let params = match set_calibrate_parameters_from_file(handle, "./CalibrateInput.txt") {
        Ok(p) => p,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::calibrate::set_calibrate_parameters_from_file' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };
    
    let min_period: f64 = params.period * params.tolerance;
    let max_period: f64 = params.period * (2.0 - params.tolerance);

    let mut hspd: u64 = match get_high_speed(handle) {
        Ok(hspd) => hspd,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::get_high_speed' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };
    let mut time: f64 = 0.0;
    let mut times: Vec<f64> = vec![0.0; params.averaging_cycles as usize];

    let pos_file = &mut File::create("./CalibrateOutput.txt").unwrap();

    match set_microstepping(handle, 16) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_microstepping' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };
    match set_movement_type(handle, "inc") {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_movement_type' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };
    match set_pulse_position(handle, 0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_pulse_position' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };
    match set_encoder_position(handle, 0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_encoder_position' failed with error {} in 'stage_control::calibrate::calibrate'", e).unwrap();
            return Err(e);
        }
    };

    while time < min_period || time > max_period {
        let total_time = Instant::now();
        for i in 0..params.averaging_cycles as usize {
            times[i] = get_cycle_time(handle, params.amplitude, pos_file, total_time, params.dwell_time)?;
        }
        
        time = _get_average_of_vector(&times);

        hspd = _adjust_speed(handle, 1, params.factor, params.period, time, hspd)?;
        
        if hspd > params.max_speed {
            println!("Max high speed tripped! Value was {}.", hspd);
            return Err(rusb::Error::Overflow);
        }
    }

    println!("Calibration complete. Parameters outputted to 'RunInput_calibrated'");

    let mut out_file = File::create("RunInput_calibrated").unwrap();
    out_file.write_all(&[
        b"HighSpeed ", hspd.to_string().as_bytes(), b"\n",
        b"LowSpeed ", params.low_speed.to_string().as_bytes(), b"\n",
        b"AccelerationTime ", params.acceleration_time.to_string().as_bytes(), b"\n",
        b"DecelerationTime ", params.deceleration_time.to_string().as_bytes(), b"\n",
        b"IdleTime ", params.idle_time.to_string().as_bytes(), b"\n",
        b"LoadStandbyCyles ", b"\n",
        b"LoadCycles ", b"\n",
        b"Amplitude ", params.amplitude.to_string().as_bytes(), b"\n",
        b"Offset ", b"\n",
        b"Period ", params.period.to_string().as_bytes(), b"\n",
        b"StandbyDuration ", b"\n",
        b"DwellTime ", params.dwell_time.to_string().as_bytes(), b"\n\n\n"].concat()).unwrap();
    
    out_file.write_all(&[b"# Calibration performed with: \n# Averaging cyles: ", params.averaging_cycles.to_string().as_bytes(), b"\n",
                            b"# Final period: ", time.to_string().as_bytes()].concat()).unwrap();
    return Ok(())
}




