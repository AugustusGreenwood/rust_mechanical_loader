use serialport::{
    Parity, DataBits, StopBits, FlowControl, SerialPort, ClearBuffer
};

const PORTNAME: &str = "/dev/ttyACM0"; // Change to COM## if on windows
const BAUDRATE: u32 = 115200;
const PARITY: Parity = Parity::None;
const DATABITS: DataBits = DataBits::Eight;
const STOPBITS: StopBits = StopBits::One;
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1000);
const FLOWCONTROL: FlowControl = FlowControl::Hardware; // Newline + Carriage return


pub fn open_string_potentiometer() -> Box<dyn SerialPort> {
    let serial_port: Box<dyn SerialPort> = serialport::new(PORTNAME, BAUDRATE)
                                                        .parity(PARITY)
                                                        .data_bits(DATABITS)
                                                        .stop_bits(STOPBITS)
                                                        .flow_control(FLOWCONTROL)
                                                        .timeout(TIMEOUT)
                                                        .open().expect("Couldn't open port");

    std::thread::sleep(std::time::Duration::from_secs(2));

    serial_port.clear(ClearBuffer::All).expect("Couldn't clear buffer");

    return serial_port;
}

pub fn get_data(mut serial_port: Box<dyn SerialPort>) -> i32 {
    let write_buffer: &[u8] = b"\n";
    let mut read_buffer: [u8; 4] = [0; 4];
    
    serial_port.write(write_buffer).expect("Couldn't write to string potentiometer (arduino)");
    serial_port.read(&mut read_buffer).expect("Couldn't read from string potentiometer (aruino)");

    let data: i32 = std::str::from_utf8(&read_buffer)
                            .unwrap()
                            .trim_matches('\0')
                            .parse().unwrap();

    return data;
}