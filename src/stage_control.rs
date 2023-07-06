mod driver; 
pub mod control;
pub mod calibrate;
pub mod run;
pub use driver::{open, close, send_command_get_response}; // There may be a better way to do this. These function are defined in the private driver file
                                                          // and rely heavy on those private fucntion but need to be exposed to the user



