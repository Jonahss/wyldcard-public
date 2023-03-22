let pngparse = require('pngparse')
let path = require('path')
let fs = require('fs/promises')
let _ = require('lodash')

const DEFAULT_IMAGE_DIRECTORY = path.resolve('/', 'home', 'pi', 'Pictures', 'wyldcard')
const DEFAULT_IMAGE_COLLECTION = 'collectionB'

let randomImage = async function(imageDirectory = DEFAULT_IMAGE_DIRECTORY, collectionName = DEFAULT_IMAGE_COLLECTION) {
  let directory = path.resolve(imageDirectory, collectionName)
  let imageNames = await fs.readdir(directory)
  let randomImageName = _.sample(imageNames)
  let randomImagePath = path.join(directory, randomImageName)
  return loadPng(randomImagePath)
}

let loadPng = async function(imagePath) {
  return new Promise((resolve, reject) => {
    pngparse.parseFile(imagePath, (err, data) => {
      if (err) { return reject(err) }

      if (data.width != 128 || data.height != 296 || data.channels != 1 || data.data.length != 37888) {
        reject(new Error('PNG image must have dimension 128x296 and a single channel with bit depth 2. Total byte length of decoded png should be 37838 pixels'))
      }

      // we then convert 37888 pixels into 9472 bytes, since each pixel is represented by 2 bits. 
      let buffer = Buffer.alloc(128*296*2/8)
      let pixelIterator = data.data.entries()
      for (let [i, pixel0] of pixelIterator) {
        let [, pixel1] = pixelIterator.next().value
        let [, pixel2] = pixelIterator.next().value
        let [, pixel3] = pixelIterator.next().value

        // this png library is a lil funky. A pixel of two bits, it outputs as a full byte, where the two bits are repeated 4 times
        pixel0 = pixel0 & 0b11000000
        pixel1 = pixel1 & 0b00110000
        pixel2 = pixel2 & 0b00001100
        pixel3 = pixel3 & 0b00000011

        let byte = pixel0 | pixel1 | pixel2 | pixel3

        buffer.writeUInt8(byte, i/4)
      }

      resolve(buffer)
    })
  })
}

module.exports = {
  DEFAULT_IMAGE_DIRECTORY,
  DEFAULT_IMAGE_COLLECTION,
  randomImage,
  loadPng,
}