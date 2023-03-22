let fs = require('fs/promises')
let path = require('path')

let { Plinth, imageUtilities } = require('./')

async function main() {
  let plinth = new Plinth()
  let image = await imageUtilities.randomImage()
  plinth.wells[0].displayImage(image)
}

main()