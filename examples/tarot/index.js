// THIS CODE DOESN'T ACTUALLY WORK YET
// it's close, just need to publish the latest version of the drivers. waiting on merging code which enables memory read/write


let fs = require('fs/promises')
let path = require('path')

let _ = require('lodash')
let gm = require('gm').subClass({ imageMagick: '7+' })

let { Plinth, imageUtilities } = require('@wyldcard/drivers')

async function main() {
  let plinth = new Plinth('devkit')
  
  let alreadyDrawn = [] // list of card image paths already drawn, to avoid duplicates
  
  let drawCard = async () => {
    let directory = path.resolve('~', 'Pictures', 'Rider-Waite Tarot', 'faces')
    let imageNames = await fs.readdir(directory)
    let randomImageName = _.sample(imageNames)
    
    while (!alreadyDrawn.includes(randomImageName)) {
      randomImageName = _.sample(imageNames)
    }
    alreadyDrawn.push(randomImageName)
    
    let randomImagePath = path.join(directory, randomImageName)
    return randomImagePath
  }

  let turnFacedown = async (well) => {
    // TODO use card memory for this
    well.faceup = false

    let cardBackPath = path.resolve('~', 'Pictures', 'Rider-Waite Tarot', 'back.png')
    let cardBack = imageUtilities.loadPng(cardBackPath)
    
    well.displayImage(cardBack)
  }

  let turnFaceup = async (well) => {
    // TODO use card memory for this
    well.faceup = true
    let imagePath = await drawCard()

    let image = imageUtilities.loadPng(randomImagePath)

    // reverse!
    if (_.random(1)) {
      image = gm(image).flip().toBuffer()
    }

    well.displayImage(image)
  }

  // for all cards on plinth, show card back
  let dealFacedown = async () => {
    await turnFacedown(plinth.well[0])
    await turnFacedown(plinth.well[1])
    await turnFacedown(plinth.well[2])
    await turnFacedown(plinth.well[3])
  }

  // button-press callback
  let flipCard = async (well) => {
    if (well.faceup) {
      return turnFacedown(well)
    } else {
      return turnFaceup(well)
    }
  }

  await dealFacedown()

  plinth.wells.forEach((well) => {
    well.onAButtonPress(flipCard(well))
    well.onBButtonPress(flipCard(well))
    well.onCButtonPress(flipCard(well))
  })
}

main()