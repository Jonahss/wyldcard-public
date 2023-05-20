[wordmark]

This is the public code repository for all things Wyldcard
----------------------------------------------------------
For an overview on what Wyldcard is, check out [the introductory blog post](https://www.wyldcard.io/blog/introducing-wyldcard).
After spending seven hours in the top position on Hacker News, I decided to crowdfund a [Wyldcard DevKit on Crowd Supply](https://www.crowdsupply.com/wyldcard/wyldcard-devkit)

[gif]


Game Development on Wyldcard
----------------------------------------------------------
If you have a Wyldcard DevKit, you can program your own functionality and implement game mechanics using JavaScript.
(You could also dive into a deeper level and program in any language you want, more on that later)

A simple Wyldcard demo that displays a random image on a card whenever a button is pressed could look like this:
```
let fs = require('fs/promises')
let path = require('path')

let { Plinth, imageUtilities } = require('@wyldcard/drivers')

async function main() {
  let plinth = new Plinth('devkit')

  let displayRandomImage = function(well) {
    return async () => {
      let image = await imageUtilities.randomImage()
      well.displayImage(image)
    }
  }

  plinth.wells.forEach((well) => {
    well.onAButtonPress(displayRandomImage(well))
    well.onBButtonPress(displayRandomImage(well))
    well.onCButtonPress(displayRandomImage(well))
  })
}

main()
```

In order to get to running this code, we need to go over the following:

- Setting Up a Development Environment
- Using the Wyldcard JavaScript SDK
- Driver Documentation

If you want to design your own Wyldcard-compatible hardware, check out:

- Hardware Documentation