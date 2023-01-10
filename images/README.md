Images moved up a few directories, outside this repo.
Images to be included on the base system should be loaded using the 'loadImages.sh' script

For reference:
Images should be:
  - .png format
  - 128 x 296 pixels
  - 4 color grayscale.

I run all images through a program which gets them to exactly the PNG format needed by the wyldcard hardware. Sometimes this changes the appearance of the image, especially when it comes to enforcing the 4-color grayscale requirement. If you use a larger pallete, some of the colors will be "rounded" up or down to the nearest grayscale value.

Also, keep in mind that the screens are nowhere near color-accurate. Their "black" is pretty black, but their "white" is more like gray.


If you want instant feedback on how your image will turn out, install [imageMagick](https://imagemagick.org/) and run it from the commandline like so:
`convert orignal_file.png -alpha off -colorspace gray -depth 2 converted_file.png`