##On dev computer

install rust
`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

for development setup, install rust-analyzer for VSCode plugin:
```
git clone git@github.com:rust-analyzer/rust-analyzer.git
cd rust-analyzer
cargo xtask install --server
```
The above builds it from source, but it might also be available from `rustup`.
See https://github.com/rust-analyzer/rust-analyzer/issues/9670

set setting for where the rust-analyze language server binary is stored.
```
{ "rust-analyzer.server.path": "~/.local/bin/rust-analyzer" }
```

Use the launch configuration in `launch.json`.
Use `CodeLLDB` extension.

## On the Raspberry Pi

For Raspberry Pi Zero W v1:

### Install Raspberry Pi OS Lite

### Enable "Gadget Mode" to SSH over USB

see https://howchoo.com/pi/raspberry-pi-gadget-mode

Insert imaged SD card, append to `config.txt`:
```
dtoverlay=dwc2
```

Enable SSH:
```
touch ssh
```

In `cmdline.txt`, after `rootwait`, add
```
modules-load=dwc2,g_ether
```

Attach pi to computer via USB. Wait for it to show up in network as an Ethernet device.

### Install Wyldcard

Install some dependencies:
```
sudo apt update
sudo apt upgrade
sudo apt install git vim silversearcher-ag
```
install rustup and follow instructions to install rust:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

clone the public repo
```
git clone https://github.com/Jonahss/wyldcard-public.git
```

navigate into the project and run
```
cargo run
```

### Enable peripheral access


https://artivis.github.io/post/2020/pi-zero/


For the rasberry pi, and this project, we may need to send more data over the SPI bus than the default buffer size.
From the docs for `rppal`:
```
By default, spidev can handle up to 4096 bytes in a single transfer. You can increase this limit to a maximum of 65536 bytes by appending spidev.bufsiz=65536 to the single line of parameters in /boot/cmdline.txt. Remember to reboot the Raspberry Pi afterwards. The current value of bufsiz can be checked with cat /sys/module/spidev/parameters/bufsiz.
```

Also enable the I2C bus using `sudo raspi-config`

Enable UART:
  Disable the Linux serial console by either deactivating it through sudo raspi-config, or manually removing the parameter console=serial0,115200 from /boot/cmdline.txt.
  On Raspberry Pi models with a Bluetooth module, an extra step is required to either disable Bluetooth or move it to /dev/ttyS0, so /dev/ttyAMA0 becomes available for serial communication
  To move the Bluetooth module to /dev/ttyS0, instead of disabling it with the above-mentioned steps, add dtoverlay=pi3-miniuart-bt and core_freq=250 to /boot/config.txt



