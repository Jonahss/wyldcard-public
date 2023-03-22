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
        reject(new Error('PNG image must have dimension 128x296 and a single channel with bit depth 2. Total byte length of decoded png should be 37838 bytes'))
      }

      resolve(data.data)
    })
  })
}

module.exports = {
  DEFAULT_IMAGE_DIRECTORY,
  DEFAULT_IMAGE_COLLECTION,
  randomImage,
  loadPng,
}