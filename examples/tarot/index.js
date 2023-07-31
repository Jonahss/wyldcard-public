let fs = require('fs/promises')
let path = require('path')

let _ = require('lodash')
let gm = require('gm').subClass({ imageMagick: true })

let { Plinth, imageUtilities } = require('@wyldcard/drivers')

// let cards be drawn upside down or 'reversed' in tarot parlance
const ENABLE_REVERSE = false

async function main() {
  let plinth = new Plinth('devkit')

  let alreadyDrawn = ['back.png'] // list of card image paths already drawn, to avoid duplicates

  let drawCard = async () => {
    let directory = path.resolve('/', 'home', 'pi', 'Pictures', 'wyldcard', 'tarot-reliberate')
    let imageNames = await fs.readdir(directory)
    let randomImageName = _.sample(imageNames)
    console.log('random image name', randomImageName, !alreadyDrawn.includes(randomImageName))
    while (alreadyDrawn.includes(randomImageName)) {
      randomImageName = _.sample(imageNames)
    }
    console.log('landed on image', randomImageName)
    alreadyDrawn.push(randomImageName)
    
    let randomImagePath = path.join(directory, randomImageName)
    return randomImagePath
  }

  let turnFacedown = async (well) => {
    well.storeData({ tarotCard: 'facedown' })

    let cardBackPath = path.resolve('/', 'home', 'pi', 'Pictures', 'wyldcard', 'tarot-reliberate', 'back.png')
    let cardBack = await imageUtilities.loadPng(cardBackPath)
    
    well.displayImage(cardBack)
  }

  let turnFaceup = async (well) => {
    well.storeData({ tarotCard: 'faceup' })

    let imagePath = await drawCard()

    let image = await imageUtilities.loadPng(imagePath)
    console.log('turning faceup, image:', imagePath)
    // reverse!
    if (ENABLE_REVERSE) {
      if (_.random(1)) {
        let reverse = new Promise((resolve, reject) => {
          gm(imagePath).flip().write('/tmp/reversed.png', function (err) {
            if (err) return reject(err)
            return resolve()
          })
        })
        await reverse
        image = await imageUtilities.loadPng('/tmp/reversed.png')
      }
    }

    well.displayImage(image)
  }

  // for all cards on plinth, show card back
  let dealFacedown = async () => {
    await turnFacedown(plinth.wells[0])
    await turnFacedown(plinth.wells[1])
    await turnFacedown(plinth.wells[2])
    await turnFacedown(plinth.wells[3])
  }

  // returns a button-press callback
  let flipCard = (well) => {
    let memory;
  
    try {
      memory = well.getData()
    } catch (e) {
      console.log(`memory isn't formatted, turning card facedown`)
      return turnFacedown(well)
    }

    if (!memory.tarotCard) {
      console.log(`card wasn't set up as a tarot card, turning facedown`)
      return turnFacedown(well)
    }

    if (memory.tarotCard == 'faceup') {
      console.log('card was faceup, turning facedown')
      return turnFacedown(well)
    } else if (memory.tarotCard == 'facedown') {
      console.log('card was facedown, turning faceup')
      return turnFaceup(well)
    } else {
      console.log(`card wasn't facedown or faceup?? turn facedown`)
      return turnFacedown(well)
    }
  }

  let reset = async () => {
    alreadyDrawn = ['back.png']
    dealFacedown()
  }

  await dealFacedown()

  let handleButtonPress = async function(event) {
    let well = plinth.wells[event.well]
    if (event.buttons.length > 1) {
      reset()
    } else {
      flipCard(well)
    }
  }

  plinth.wells.forEach((well) => {
    well.on('chordedButtonPress', handleButtonPress)
  })
}

main()