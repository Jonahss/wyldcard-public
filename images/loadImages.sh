# ssh pi@raspberrypi.local "rm -rf /home/pi/Pictures/wyldcard"
# ssh pi@raspberrypi2.local "rm -rf /home/pi/Pictures/wyldcard"

# ssh pi@raspberrypi.local "mkdir -p /home/pi/Pictures/wyldcard"
# ssh pi@raspberrypi2.local "mkdir -p /home/pi/Pictures/wyldcard"

# scp -r ../../../art/converted/collectionB pi@raspberrypi.local:/home/pi/Pictures/wyldcard/
# scp -r ../../../art/converted/collectionB pi@raspberrypi2.local:/home/pi/Pictures/wyldcard/

ssh pi@raspberrypizero.local "rm -rf /home/pi/Pictures/wyldcard"
ssh pi@raspberrypizero.local "mkdir -p /home/pi/Pictures/wyldcard"
scp -r ../../../art/converted/collectionB pi@raspberrypizero.local:/home/pi/Pictures/wyldcard