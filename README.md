# Disclaimer:
I am not affiliated with NewMark System or Arcus Technologies at all and this code does not represent them. It is heavily influenced by their own implementation written in C [Arcus Technology C-driver](https://www.arcus-technology.com/support/downloads/download-info/linux-usb-source/). I provide no warranties or guarantees for anything. 

# Dependencies:
Technically none, but you *might* need libusb-1.0
rusb claims that libusb installation is not needed, proceed at own risk if you don't have it installed. I have only ever used rusb with it installed

# Compilation:
First get cargo and the Rust tool chain.
Navigate to top directory and: 
$ cargo build

- Linux:
For libusb, on Debian $ sudo apt-get install libusb-dev
              On Arch $ sudo pacman -S libusb
- Windows:
I used MSYS2 and was able to get it to work.
For libusb, install MSYS2, then "pacman -S mingw-w64-x86_64-libusb". You may need to add MSYS2 to your path. After everything, you should be able to run "cargo build --release" to be finished. WSL may be helpful in this regard


# Troubleshooting
- If you ever send a move command and it just sputters and doesn't move smoothly, you likely need to increase the run current (DRVIC=\[100-3000\]). It doesn't have sufficient torque to move.
- If it get way to hot just sitting there you have two options:
    * Reduce the idle current. This decreases the torque it can handle, so choose wisely
    * Turn the motor off. Obviously this only works if it's not holding anything and friction can keep it in place
- If it gets too hot for your environment running, then you can reduce the run current but be wary of the torque needed and that the motor runs smoothly
- This program is as robust as I had the time to make, it should correctly release everything and so no problems should arise from normal use. If you keep getting TIMEOUT errors, you likely just need to restart the device (power cycle) and it should work again. I (so far) haven't ever broken it so bad I had to do more than that.
- I want to add a help command to output the commands possible in interactive mode, but thats a lot of work. In the mean-time, here is the link to the manual with all the commands. They start at section 10 on page 55. [NSC-A1 user manual](https://www.newmarksystems.com/downloads/software/NSC-A/NSC-A1/NSC-A1_Manual_Rev_1.3.0.pdf)
- To see where errors come from, run: $ RUST_BACKTRACE=1 cargo run
- the "calibrate" and "run" commands were build very specifically to my needs so I can't image they would be useful to anyone else unless you are cyclically loading with a trapezoidal waveform. However, the commands and interactive mode would likely be very useful. 
- I want to add a GUI, but Rust just isn't great with GUIs. Maybe will add a GTK interface but I wouldn't count on it.
- If it move way to roughly or vibrates like crazy, changing microsteps can be a good way to reduce these things. Be aware that this will change the distance moved with a pulse so you may need to recalibrate
