let EventEmitter = require('events')
let util = require('util')

let _ = require('lodash')

let { JsPrototype, JsDevkit } = require('./nativeBinding')

class Plinth extends EventEmitter {
  constructor(model = 'devkit') {
    super()

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

class Well extends EventEmitter {
  constructor(id, plinth) {
    super()

    this.id = id
    this.plinth = plinth
    this.maxMemory = 4096 // bytes. basically 4kb
    this.dimensions = {
      x: 128,
      y: 296,
    }

    this.buttonPressBuffer = new Map() // for detecting chorded button presses
    this.chordTimeout = 35 // amount of time in milliseconds between button presses which will count as being pressed at the same time to form a chord
    this.endOfChordTimer = setTimeout(()=>{}, 1)
    this.on('buttonPress', this._emitChordedButtonPress)
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
    let text = JSON.stringify(object)
    if (text.length > this.maxMemory) {
      throw new Error(`attempted to store too much data. JSON stringified data is of length ${text.length}, which is more than the maximum of ${this.maxMemory}. You could try using the _writeMemory and _readMemory functions directly, storing data in binary rather than ascii JSON.`)
    }

    let buf = Buffer.alloc(4096, ' ')
    buf.write(text)

    this._writeMemory(buf)
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
    validateCallback(cb)
    cb = this._wrapCallbackToEmitEvents(cb, 'a')
    this.plinth.setSwitchCallback(this.id, 'a', cb)
  }

  // register a callback to be called when Switch B (the middle button) for this well is pressed
  onBButtonPress = function(cb) {
    validateCallback(cb)
    cb = this._wrapCallbackToEmitEvents(cb, 'b')
    this.plinth.setSwitchCallback(this.id, 'b', cb)
  }
  
  // register a callback to be called when Switch C (the bottom button) for this well is pressed
  onCButtonPress = function(cb) {
    validateCallback(cb)
    cb = this._wrapCallbackToEmitEvents(cb, 'c')
    this.plinth.setSwitchCallback(this.id, 'c', cb)
  }

  _wrapCallbackToEmitEvents = function(cb, buttonId) {
    let well = this

    return async () => {
      well.emit('buttonPress', {well: well.id, button: buttonId, ts: Date.now()})
      cb()
    }
  }

  _emitChordedButtonPress = function(buttonPressEvent) {
    let well = this
    clearTimeout(well.endOfChordTimer)
    let chordTimeout = well.chordTimeout
    well.buttonPressBuffer.set(buttonPressEvent.button, buttonPressEvent.ts)

    let recentPresses = _.filter(Array.from(this.buttonPressBuffer.entries()), ([button, ts]) => ts > buttonPressEvent.ts - chordTimeout)
    recentPresses = recentPresses.map(([button, ts]) => button)

    well.endOfChordTimer = setTimeout(() => {
      well.emit('chordedButtonPress', { well: well.id, buttons: recentPresses })
      well.buttonPressBuffer = new Map()
    }, chordTimeout)
  }
}

function validateCallback(cb) {
  if (!util.types.isAsyncFunction(cb)) {
    throw new Error('callbacks passed to `onXButtonPress()` handlers must be async functions')
  }
}

module.exports = {
  Plinth
}