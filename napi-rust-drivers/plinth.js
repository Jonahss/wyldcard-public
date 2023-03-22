let { JsPlinth } = require('./nativeBinding')

class Plinth {
  constructor() {
    this.plinth = new JsPlinth()
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
}

module.exports = {
  Plinth
}