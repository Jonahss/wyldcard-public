let fs = require('fs/promises')
let path = require('path')

let { Plinth, imageUtilities } = require('./')

async function main() {
  let plinth = new Plinth('prototype')
  let image = await imageUtilities.randomImage()
  //let image = await imageUtilities.loadPng('/home/pi/Workspace/wyldcard-public/images/converted/Peacock.png')
  plinth.wells[0].displayImage(image)
}

main()