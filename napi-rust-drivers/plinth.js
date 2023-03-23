let { JsPrototype } = require('./nativeBinding')

class Plinth {
  constructor(model = 'devkit') {
    if (model == 'prototype') {
      this.plinth = new JsPrototype()
    }
    else if (model == 'devkit') {
      this.plinth = new Devkit()
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
  }

  displayImage = async function(imageBuffer) {
    this.plinth.displayImage(this.id, imageBuffer)
  }

  onAButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'a', cb)
  }
  onBButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'b', cb)
  }
  onCButtonPress = function(cb) {
    this.plinth.setSwitchCallback(this.id, 'c', cb)
  }
}

module.exports = {
  Plinth
}