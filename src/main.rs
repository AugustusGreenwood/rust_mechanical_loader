mod stage;

use stage::*;

fn main() -> rusb::Result<()> {
    let handle = &mut open()?;
    
    //interactive_mode(handle)?;
    //calibrate_time(handle)?;
    sin_run(handle, 10800.0)?;
    //camel_run(handle, 1800.0)?;

    close(handle)?;
    Ok(())
}