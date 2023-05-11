// let fs = require('fs/promises')
// let path = require('path')

// let { Plinth, imageUtilities } = require('.')

// async function main() {
//   let plinth = new Plinth('prototype')


//   let displayRandomImage = function(well) {
//     return async () => {
//       let image = await imageUtilities.randomImage()
//       plinth.wells[well].displayImage(image)
//     }
//   }

//   let getData = function(well) {
//     return async () => {
//       let data = well.getData()
//       console.log('data for card in well', well.id, data)
//     }
//   }

//   plinth.wells.forEach((well, i) => {
//     well.onAButtonPress(getData(well))
//     well.onBButtonPress(getData(well))
//     well.onCButtonPress(getData(well))
//   })
// }

// main()

let fs = require('fs/promises')
let path = require('path')

let { Plinth, imageUtilities } = require('./')

async function main() {
  let plinth = new Plinth('prototype')


  let displayRandomImage = function(well) {
    return async () => {
      let image = await imageUtilities.randomImage()
      plinth.wells[well].displayImage(image)
    }
  }

  let getData = function(well) {
    return () => {
      let data = well.getData()
      return data
    }
  }

  let erase = function(well) {
    console.log('erasing well', well.id)
    let buf = Buffer.alloc(4096, ' '.charCodeAt(0))
    buf[0] = '{'.charCodeAt(0)
    buf[1] = '}'.charCodeAt(0)
    well._writeMemory(buf)
    console.log('erased well', well.id)
  }

  let logCallbackPress = function(well, buttonId) {
    return async () => {
      console.log('logging button press by callback: well', well.id, buttonId)
    }
  }

  let logEventPress = function(well) {
    return (e) => {
      console.log('logging button press by event: well', e.well, e.button)
    }
  }

  let logEventChordedPress = function(well) {
    return (e) => {
      console.log('logging chorded button press by event: well', e.well, e.buttons)
    }
  }

  plinth.wells.forEach((well, i) => {
    well.onAButtonPress(logCallbackPress(well, 'a'))
    well.onBButtonPress(logCallbackPress(well, 'b'))
    well.onCButtonPress(logCallbackPress(well, 'c'))
    well.on('buttonPress', logEventPress(well))
    well.on('chordedButtonPress', logEventChordedPress(well))
  })
}

main()