use std::time::Instant;

use rusb::{DeviceHandle, GlobalContext};
use stage_control::{
    commands::{
        close, get_pulse_position, interactive_mode, move_cycle_get_time, move_stage, open,
        wait_for_motor_idle,
    },
    driver::{read_from_bulk, saftey_read, write_to_bulk},
};

use crate::stage_control::commands::send_command_get_response;

pub mod cli;
pub mod stage_control;

fn do_stuff(handle: &DeviceHandle<GlobalContext>) {
    saftey_read(handle).unwrap();
    write_to_bulk(handle, b"PX\0").unwrap();
    let a = read_from_bulk(handle).unwrap();
    println!("{}", a);
}

// 0x1589, 0xa101
fn main() {
    let mut handle = open(0x1589, 0xa101).unwrap();

    interactive_mode(&handle).unwrap();

    let time = Instant::now();
    for _ in 1..10001 {
        do_stuff(&handle);
    }
    println!("{}", time.elapsed().as_secs_f64() / 10000.0);

    close(&mut handle).unwrap();
}
