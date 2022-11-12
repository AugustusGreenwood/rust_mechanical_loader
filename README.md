# Rust_Mechanical-Loader

"input.txt"
    Gives the system the values wanted for a movement, makes it much easier then just rewriting in code and 
    rebuilding. Use "_get_set_params_from_file"

    Distance => The total amount wanted to move in pulses
    Cycles => Amount of cycles to average for a period time
    MaxSpeed => The highest high speed before calibration will stop, makes sure the machine doesn't keep getting faster to infinity
    Factor => Mulitples the error value which adjust the high speed every time. Can keep it from moving too little or too much. 
    Tolerance => The fraction that the cycle time must be measure before calibration stops. ie if you want a 10 second period, with a .9 tolerance, calibration would stop if a cycle time is between 9s and 11s
    Period => Desired period time in seconds
    HighSpeed => beginning high speed
    LowSpeed => low speed (doesn't change)
    AccelerationTime => time to get from low speed to high speed in ms
    AccelerationProfile => must be ['sin', 'SIN', 'trap', 'TRAP'] dictates how the motor accelerates. either with constant accel or variable 
    DecelerationTime => time to get from high spee to low speed
    IdleTime => Time before the motor says it is idle, in cs