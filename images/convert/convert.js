const fs = require('fs')
const path = require('path')
const gm = require('gm').subClass({ imageMagick: '7+' });
let glob = require('glob-promise')

async function main() {
  let imageFileNames = await glob('/Users/jonahss/Workspace/wyldcard/art/DALLE3/*')

  let images = imageFileNames.map((filePath) => {
    return {
      path: filePath,
      name: path.basename(filePath),
      readStream: fs.createReadStream(filePath)
    }
  })

  console.log(`Converting ${images.length} files: `)

  for (image of images) {
    console.log(`converting: ${image.name}`)
    gm(image.readStream)
    .resize(128, 296, '^')
    .gravity('Center')
    .crop(128, 296)
    .colorspace('gray')
    .bitdepth(2)
    //.out('-alpha off')
    .stream(function (err, stdout, stderr) {
      var writeStream = fs.createWriteStream(`/Users/jonahss/Workspace/wyldcard/art/output/${image.name}`);
      stdout.pipe(writeStream)
    })
  }
}

main()