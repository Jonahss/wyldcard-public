## Developing Wyldcard applications on your Devkit ##

The Wyldcard Devkit contains a raspberry pi zero 2, on which you can run your entire development environment. This makes is easier to iterate on game design, no need to write then then push it to the device, you can change code and immediately run it and see how it behaves.

Of course, you can do anything you like with the Raspberry Pi, but I believe this is a nice setup.

The Raspberry Pi Zero 2 provided with the DevKit has the hostname: `wyldcardDevkit.local`

## Connecting to your Raspberry Pi for the first time

The goal with this setup is to get your Raspberry Pi connected to your local Wifi. Once this is set up, it will auto-connect when it powers on and you can connect VS Code and develop your game logic remotely.

There are three ways to set the Wifi credentials, ordered here from easiest and most-error-prone to awkward but failproof.

### Direct USB connection

If this happens to work, you don't need any extra materials. Use a Micro USB cable and connect the Raspberry Pi's *second* USB port to your development computer. Make sure the USB cable can carry data, cheap ones only provide power. Notice the Raspberry Pi has two USB ports, one labeled `PWR` and one labelled `USB`. Use the `USB` one.

Your computer should recognize a network connection to the device over ethernet. This somtimes requires massaging your operating system and networking configuration, and if it doesn't work you should skip to the next method.

To check if it's working, the command `ping wyldcardDevkit.local` will return packets.

If it's working, ssh into the pi

`ssh wyldcardDevkit.local`

Using password: `raspberry`

Use `sudo raspi-config` to setup your wifi connection.

Done!

Next time you use the DevKit, just connect it to power and you can connect to `wyldcardDevkit.local` as long as you're on the same wifi network.

### Using the SD Card

There's a special file on the SD card in the Raspberry Pi which you can use to set the Wifi credentials on boot. This approach requires that your development computer has a Micro SD card reader or suitable adapter.

Disconnect the usb cable connected to the Raspberry Pi Zero 2 in your Wyldcard Devkit and gently pry it away from the circuit board that forms the top surface of your Wyldcard plinth. There are 40 pins holding it in place, so it takes some easy pressure. Now that you've taken it off, you can access the SD card slot (sorry about that, next time I'll provide more clearance). Take out the SD card and plug it into your development computer.

Open your file browser and open the SD card.

We want to create a file named `wpa_supplicant`. I've included a template for you already which you can copy:

`cp SETUP_WIFI_Template ./wpa_supplicant`

Edit the `wpa_supplicant` file and fill in your specific values.

Now eject the SD card, reinsert it into the Raspberry Pi, push the Pi back onto the Wyldcard Devkit (oriented in the direction which makes it hard to reach the SD card -_-), reconnect power and after 30-60 seconds test the connection over wifi from your development computer:

`ping wyldcardDevkit.local`

If packets are returned, ssh into the pi

`ssh wyldcardDevkit.local`

Using password: `raspberry`

### Use the Raspberry Pi as a computer

If all else fails, you can always connect the Raspberry Pi in your Devkit to a keyboard and monitor and use it like a regular computer. This method is the most annoying because you need a keyboard and a monitor and a bunch of annoying adapters which I never seem to have at hand.

Follow the steps in the previous section for removing the Pi from the Devkit so you can get at all the ports.

Once the Pi boots up and you have a terminal, run

`sudo raspi-config`

to setup your wifi connection.

## Connect VS Code

Now you can use Visual Studio Code to develop on the Raspberry Pi on your Wyldcard Devkit! Use the hostname `wyldcardDevkit.local` and open one of the project directories in `/home/pi` such as `simple-demo`

## Setting the default Wyldcard application

When your devkit first boots, it automatically runs a Wyldcard application. Only one Wyldcard application can run at a time, so if you want to run a new application you're working on, you'll have to stop this automatic one:

`sudo systemctl stop wyldcard`

Now you can navigate to one of the example projects and run it:

`cd /home/pi/tarot`
`node .`

And the devkit should be running your app.

If you're developing more than you're demonstrating, you can disable the auto-start:

`sudo systemctl disable wyldcard`

If you want to change which Wyldcard app runs automatically, edit the wyldcard.service file. This file is symlinked in `/home/pi` or can be found in `/lib/systemd/system`.

In this file, change `WorkingDirectory` to your project's directory.

Remember to enable it:

`sudo systemctl enable wyldcard`

## Creating your own project directory

For your own Wyldcard application, just make a new directory, create a node.js project and add a dependency on `@wyldcard/drivers`.

Now your ready to read about the [Wyldcard Javascript SDK](/docs/using-the-wyldcard-javascript-sdk.md) or follow a [Tutorial](https://www.wyldcard.io/blog/implementing-a-tarot-deck-on-wyldcard)