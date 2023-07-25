/* 
Function for running the device. See "calibrate.rs" as there is some overlap
and more comments there if you are confused on some functions. 
*/

use crate::stage_control::control::{
    set_high_speed,
    set_low_speed,
    set_acceleration_time,
    set_deceleration_time,
    set_idle_time,
    wait_for_motor_idle,
    move_stage,
    set_microstepping,
    set_movement_type,
    set_pulse_position,
    set_encoder_position,
    get_high_speed,
    write_driver_settings,
    turn_motor_on,
    get_cycle_time
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
pub struct RunParameters {
    high_speed: u64,
    low_speed: u64, 
    acceleration_time: u64, 
    deceleration_time: u64,
    idle_time: u64,
    pub load_standby_cyles: u64,
    load_cycles: u64,
    amplitude: i64,
    offset: i64,
    period: f64,
    pub standby_duration: u64,
    dwell_time: f64
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


fn _initialize_run_parameters() -> RunParameters {
    return RunParameters{high_speed: 0u64,
                        low_speed: 0u64, 
                        acceleration_time: 0u64, 
                        deceleration_time: 0u64,
                        idle_time: 0u64,
                        load_standby_cyles: 0u64,
                        load_cycles: 0u64,
                        amplitude: 0i64,
                        offset: 0i64,
                        period: 0f64,
                        standby_duration: 0u64,
                        dwell_time: 0f64};
}

fn _get_average_of_vector(vec: &Vec<f64>) -> f64 {
    let mut sum: f64 = 0.0;
    for ele in vec {
        sum = sum + ele;
    }

    return sum / vec.len() as f64;
}

fn _adjust_speed(handle: &mut DeviceHandle<GlobalContext>, index: u64, factor: f64, period: f64, time: f64, hspd: u64) -> rusb::Result<u64> {
    let error = time - period * (index as f64);

    let new_hspd = hspd as i64 + ((error * factor * 1000.0) as i64);

    match set_high_speed(handle, new_hspd as u64) {
        Ok(()) => return Ok(new_hspd as u64),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_high_speed' failed with error {} in 'stage_control::run::_adjust_speed'", e).unwrap();
            return Err(e)
        },
    };
}




pub fn set_run_parameters_from_file(handle: &mut DeviceHandle<GlobalContext>, file_path: &str) -> rusb::Result<RunParameters> {
    let mut params: RunParameters = _initialize_run_parameters();
    
    let lines = read_file_to_vector_of_lines(file_path).expect("Couldn't read inputs to run device");
    
    for whole_line in lines {
        let line: Vec<&str> = whole_line.split_whitespace().collect();
        
        if whole_line.is_empty() || line[0].starts_with("#") {continue;}

        match line[0].to_ascii_lowercase().as_str() {
            "highspeed" => {
                match set_high_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_high_speed' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };
                params.high_speed = line[1].parse().unwrap();
            },
            "lowspeed" => {
                match set_low_speed(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_low_speed' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };
                params.low_speed = line[1].parse().unwrap();
            },
            "accelerationtime" => {
                match set_acceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_acceleration_time' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };  
                params.acceleration_time = line[1].parse().unwrap();
            },
            "decelerationtime" => {
                match set_deceleration_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_deceleration_time' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };
                params.deceleration_time = line[1].parse().unwrap();
            },
            "idletime" => {
                match set_idle_time(handle, line[1].parse::<u64>().unwrap()) {
                    Ok(()) => (),
                    Err(e) => {
                        writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_idle_time' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap();
                        return Err(e);
                    }
                };  
                params.idle_time = line[1].parse().unwrap();
            },

            "loadstandbycyles" => params.load_standby_cyles = line[1].parse().unwrap(),
            "amplitude" => params.amplitude = line[1].parse().unwrap(),
            "offset" => params.offset = line[1].parse().unwrap(),
            "period" => params.period = line[1].parse().unwrap(),
            "loadcycles" => params.load_cycles = line[1].parse().unwrap(),
            "standbyduration" => params.standby_duration = line[1].parse().unwrap(),
            "dwelltime" => params.dwell_time = line[1].parse().unwrap(),

            _ => println!("Couldn't understand {:?}", line[0].to_ascii_lowercase().as_str()),
        }
    }
    match write_driver_settings(handle) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::write_driver_settings' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap(),

    };
    match turn_motor_on(handle) {
        Ok(()) => (),
        Err(e) => writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::turn_motor_on' failed with error {} in 'stage_control::run::set_run_parameteres_from_file'", e).unwrap(),

    }; 
    return Ok(params);
}


pub fn run_prep(handle: &mut DeviceHandle<GlobalContext>) -> rusb::Result<RunParameters> {
    match set_microstepping(handle, 16) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_microstepping' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };
    match set_movement_type(handle, "inc") {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_movement_type' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };
    match set_pulse_position(handle, 0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_pulse_position' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };
    match set_encoder_position(handle, 0) {
        Ok(()) => (),
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::set_encoder_position' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };

    let params = match set_run_parameters_from_file(handle, "./RunInput.txt") {
        Ok(p) => p,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::run::set_run_parameters_from_file' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };

    return Ok(params)
}



// So, the most important thing is making sure that long term, the device stays on the correct period.
// If the high speed isn't changed, then over many many cycles, what was a 0.01 difference on the 
// first cycle, propogates to be a 10 seconds on cycle 1000 (which we reasonable reach). This is also why we 
// can't adjust the speed based on a single cycle. We may be very accurate for a single cycle time, 
// but the overall error may be too big. 
// This is why we need the 'index' in '_adjust_high_speed' function. In a perfect world, the device 
// finished a cycle on period * index, so we use this and the total time running to calculate the error. 
pub fn run(handle: &mut DeviceHandle<GlobalContext>, params: &RunParameters, file: &mut File) -> rusb::Result<()> {
    let start_hspd: u64 = match get_high_speed(handle) {
        Ok(hspd) => hspd,
        Err(e) => {
            writeln!(std::io::stderr(), "ERROR: Function 'stage_control::control::get_high_speed' failed with error {} in 'stage_control::run::run'", e).unwrap();
            return Err(e);
        }
    };

    set_movement_type(handle, "abs")?;
    set_high_speed(handle, 1000)?;
    move_stage(handle, 0)?;
    set_movement_type(handle, "inc")?;
    wait_for_motor_idle(handle, None, None)?;

    set_high_speed(handle, start_hspd)?;
    move_stage(handle, -params.offset)?;
    wait_for_motor_idle(handle, None, None)?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    let total_time = Instant::now();
    for load_cycle in 1..params.load_cycles+1 {
        get_cycle_time(handle, params.amplitude, file, total_time, params.dwell_time)?;
        _adjust_speed(handle, load_cycle, 2.0, params.period, total_time.elapsed().as_secs_f64(), start_hspd)?;
    }

    move_stage(handle, params.offset + 4913)?;
    wait_for_motor_idle(handle, None, None)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    return Ok(());
}