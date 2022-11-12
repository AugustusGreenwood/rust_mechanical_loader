mod stage;

use stage::*;
use std::io::{Read, Write};


fn main() -> rusb::Result<()> {
    let handle = &mut open()?;

    calibrate_time(handle)?;
    
    close(handle)?;
    Ok(())
}