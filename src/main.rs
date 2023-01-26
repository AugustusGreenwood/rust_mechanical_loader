mod stage;

use stage::{open, interactive_mode, close, calibrate_time, run};

fn main() -> rusb::Result<()> {
    let handle = &mut open()?;
    
    interactive_mode(handle)?;
    //calibrate_time(handle)?;
    //run(handle)?;
    //run(handle)?;
    close(handle)?;
    Ok(())
}
