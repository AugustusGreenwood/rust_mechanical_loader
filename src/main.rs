use cli::cli::cli;

pub mod cli;
pub mod stage_control;

// 0x1589, 0xa101
fn main() {
    cli().unwrap();
}
