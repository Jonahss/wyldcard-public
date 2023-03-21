let fs = require('fs/promises')
let path = require('path')

let wyldcard = require('./')

async function main() {
  let plinth = new wyldcard.JsPlinth()

  console.log('plinth', plinth)

  let pathToImage = path.resolve('..', 'images', 'converted', 'lospec_0.png')

  let imageData = await fs.readFile(pathToImage)

  console.log(imageData.BYTES_PER_ELEMENT, imageData.byteLength, imageData.length)

  plinth.displayImage(0, imageData)
}

main()