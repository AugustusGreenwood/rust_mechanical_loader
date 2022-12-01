mod stage;

use stage::*;

fn main() -> rusb::Result<()> {
    let handle = &mut open()?;
    
    interactive_mode(handle)?;
    run(handle)?;

    close(handle)?;
    Ok(())
}
