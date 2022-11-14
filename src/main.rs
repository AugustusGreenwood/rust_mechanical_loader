mod stage;

use stage::*;

fn main() -> rusb::Result<()> {
    let handle = &mut open()?;

    run(handle, 10800.0)?;
    
    close(handle)?;
    Ok(())
}