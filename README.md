# Dependencies:
libusb (rust equivalen is rusb), which provides the way to interface with the device.
###

# Compilation:
Must have libusb!


On linux:
For libusb, on Debian # sudo apt-get install libusb-dev
              On Arch # sudo pacman -S libusb

After getting libusb, to compile with cargo is easy, simply navigate to the overall directory and run: "cargo build --release". This will automatically build. 

On windows:
I used the MSYS2 way and was able to get it to work.
For libusb, install MSYS2, then "pacman -S mingw-w64-x86_64-libusb". You may need to add MSYS2 to your path. After everything, you should be able to run "cargo build --release" to be finished.
