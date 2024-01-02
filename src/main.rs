pub mod stage_control;
pub mod cli;

// 0x1589, 0xa101
fn main() {
    cli::cli::main_loop().unwrap();
}
