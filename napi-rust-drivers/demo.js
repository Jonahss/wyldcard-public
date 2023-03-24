let fs = require('fs/promises')
let path = require('path')

let { Plinth, imageUtilities } = require('.')

async function main() {
  let plinth = new Plinth('prototype')


  let displayRandomImage = function(well) {
    return async () => {
      let image = await imageUtilities.randomImage()
      plinth.wells[well].displayImage(image)
    }
  }

  plinth.wells.forEach((well, i) => {
    well.onAButtonPress(displayRandomImage(i))
    well.onBButtonPress(displayRandomImage(i))
    well.onCButtonPress(displayRandomImage(i))
  })
}

main()