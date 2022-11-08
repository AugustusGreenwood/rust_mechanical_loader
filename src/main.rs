
fn main() {
    loop {
        let mut command: String = String::new();
        std::io::stdin()
            .read_line(&mut command)
            .expect("Couldn't read line");

        if command.trim() == "EXIT" {
            break;
        }
        println!("{}", command.trim());
    }
}