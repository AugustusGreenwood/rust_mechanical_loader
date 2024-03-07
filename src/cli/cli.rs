use crate::stage_control::{
    calibrate::calibrate,
    commands::{close, interactive_mode, open},
    run::run,
};

use std::io::{stdin, Write};

// 0x1589, 0xa101
pub fn cli() -> rusb::Result<()> {
    let handle = open(0x1589, 0xa101)?;

    loop {
        let mut raw_input = String::new();
        writeln!(
            std::io::stdout(),
            "Entering main loop. Enter 'calibrate', 'run' or 'interact'."
        )
        .unwrap();

        match stdin().read_line(&mut raw_input) {
            Ok(_n) => (),
            Err(e) => writeln!(std::io::stderr(), "Failed to read line with error {}", e).unwrap(),
        }

        let input = raw_input.trim().to_ascii_lowercase();

        match input.as_str() {
            "exit" => break,
            "run" => run(&handle)?,
            "calibrate" => calibrate(&handle)?,
            "interact" => interactive_mode(&handle)?,
            _ => writeln!(
                std::io::stderr(),
                "Didn't understand '{}'. Enter 'run', 'calibrate', 'interact' or 'exit'",
                input
            )
            .unwrap(),
        };
    }

    close(&handle)?;
    Ok(())
}
