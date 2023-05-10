let fs = require('fs/promises')
let path = require('path')
let { createCanvas } = require('canvas');

let { Plinth } = require('@wyldcard/drivers')

// return the button press callback, associated with a well and button
function buttonPress(well, buttonsPressed) {
  return async () => {
    memoryTemplate = {
      buttonPresses: [[],[],[],[],[]],
    }

    let memory;
    
    try {
      memory = well.getData()
    } catch (e) {
      console.log('memory read error:', e)
      well.storeData(memoryTemplate)
      memory = memoryTemplate
    }

    if (!memory.buttonPresses) {
      console.log('did not detect button presses data, erasing memory and starting fresh')
      well.storeData(memoryTemplate)
      memory = memoryTemplate
    }

    memory.buttonPresses.pop()
    memory.buttonPresses.unshift(buttonsPressed)

    console.log('memory for card in well', well.id, memory)
    well.storeData(memory)

    let image = render(memory.buttonPresses)
    //console.log(image)
    well.displayImage(image)
  }
}

function render(buttonPresses) {
  let canvas = createCanvas(128, 296)
  let ctx = canvas.getContext('2d', { pixelFormat: 'A8' })

  ctx.fillStyle = 'black'

  let margin = 3

  buttonPresses.forEach((press, i) => {
    let y = (296 / 8 * 5) - 6
    let height = 39
    let width = 25
    let x = i*width+1
    if (press.includes('A')) {
      ctx.fillRect(x+margin, y+margin, width-2*margin, height-2*margin)
    }
    if (press.includes('B')) {
      ctx.fillRect(x+margin, y+height+margin, width-2*margin, height-2*margin)
    }
    if (press.includes('C')) {
      ctx.fillRect(x+margin, y+height+height+margin, width-2*margin, height-2*margin)
    }
  })

  let pixels = canvas.toBuffer('raw')
  // Buffer of length 37888, one byte representing each pixel.
  let pixelsFormattedForWyldcard = formatPixelBuffer(pixels)

  return pixelsFormattedForWyldcard
}

function formatPixelBuffer(raw) {
  // we then convert 37888 pixels into 9472 bytes, since each pixel is represented by 2 bits. 
  let buffer = Buffer.alloc(128*296*2/8)
  let pixelIterator = raw.entries()
  for (let [i, pixel0] of pixelIterator) {
    let [, pixel1] = pixelIterator.next().value
    let [, pixel2] = pixelIterator.next().value
    let [, pixel3] = pixelIterator.next().value

    pixel0 = pixel0 & 0b11000000
    pixel1 = pixel1 & 0b00110000
    pixel2 = pixel2 & 0b00001100
    pixel3 = pixel3 & 0b00000011

    let byte = pixel0 | pixel1 | pixel2 | pixel3

    byte = ~byte & 0xFF // invert because 0xff is white in hex color land, but 0b11 is "on" pixels for us and therefore black

    buffer.writeUInt8(byte, i/4)
  }

  return buffer
}

async function main() {
  let plinth = new Plinth('devkit')

  plinth.wells.forEach((well) => {
    well.onAButtonPress(buttonPress(well, ['A']))
    well.onBButtonPress(buttonPress(well, ['B']))
    well.onCButtonPress(buttonPress(well, ['C']))
  })
}

main()