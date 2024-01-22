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

  let getData = function(well) {
    return async () => {
      let data = well.getData()
      console.log('data for card in well', well.id, data)
    }
  }

  plinth.wells.forEach((well) => {
    well.onAButtonPress(displayRandomImage(well))
    well.onBButtonPress(displayRandomImage(well))
    well.onCButtonPress(displayRandomImage(well))
  })
}

main()



// catch all exceptions for this demo
process.on('uncaughtException', (err) => {
  console.error(err);
})