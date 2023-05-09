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
      console.log('data for card in well', well.id, data)
    }
  }

  let erase = function(well) {
    console.log('erasing well', well.id)
    let buf = Buffer.alloc(4096, ' '.charCodeAt(0))
    buf[0] = '{'.charCodeAt(0)
    buf[1] = '}'.charCodeAt(0)
    console.log(buf.toString())
    well._writeMemory(buf)
    console.log('erased well', well.id)
  }

  // plinth.wells.forEach((well, i) => {
  //   well.onAButtonPress(getData(well))
  //   well.onBButtonPress(getData(well))
  //   well.onCButtonPress(getData(well))
  // })

  erase(plinth.wells[0])
  console.log('erased. now getting data')
  getData(plinth.wells[0])()

  plinth.wells[0].storeData({ hello: 'chukwudi'})
  getData(plinth.wells[0])()
}

main()