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

  let getData = function(well) {
    return async () => {
      let data = well.getData()
      console.log('data for card in well', well.id, data)
    }
  }

  plinth.wells.forEach((well, i) => {
    well.onAButtonPress(getData(well))
    well.onBButtonPress(getData(well))
    well.onCButtonPress(getData(well))
  })
}

main()