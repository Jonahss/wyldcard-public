let { JsPrototype, JsDevkit } = require('./nativeBinding')

class Plinth {
  constructor(model = 'devkit') {
    if (model == 'prototype') {
      this.plinth = new JsPrototype()
    }
    else if (model == 'devkit') {
      this.plinth = new JsDevkit()
    }
    else {
      throw new Error('must supply argument to Plinth constructor, either "prototype" or "devkit"')
    }
    
    this.wells = [
      new Well(0, this.plinth),
      new Well(1, this.plinth),
      new Well(2, this.plinth),
      new Well(3, this.plinth),
    ]
  }
}

class Well {
  constructor(id, plinth) {
    this.id = id
    this.plinth = plinth
    this.maxMemory = 4096 // bytes. basically 4kb
  }

  // display an image on the e-paper display of the wyldcard present in this well
  // pass in a Buffer. You probably want to create this using the methods in `imageUtilities`
  displayImage = async function(imageBuffer) {
    this.plinth.displayImage(this.id, imageBuffer)
  }

  // write raw bytes to the memory chip within the wyldcard present in this well
  // it's easier to use the `storeData()` method, which serializes a javascript object for you
  // this method takes a raw Buffer of bytes 
  _writeMemory = function(buffer) {
    this.plinth.writeMemory(this.id, buffer)
  }

  storeData = function(object) {
    this._writeMemory(Buffer.from(JSON.stringify(object)))
  }

  _readMemory = function() {
    return this.plinth.readMemory(this.id, this.maxMemory)
  }

  getData = function() {
    let text = this._readMemory().toString().trim()
    return JSON.parse(text)
  }

  // register a callback to be called when Switch A (the top button) for this well is pressed
  onAButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'a', cb)
  }

  // register a callback to be called when Switch B (the middle button) for this well is pressed
  onBButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'b', cb)
  }
  
  // register a callback to be called when Switch C (the bottom button) for this well is pressed
  onCButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'c', cb)
  }
}

module.exports = {
  Plinth
}