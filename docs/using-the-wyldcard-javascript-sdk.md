# Using the Wyldcard JavaScript SDK

Install the javascript module
```
npm install @wyldcard/drivers
```

Import the sdk
```
let { Plinth } = require('@wyldcard/drivers')
```

construct a plinth, which represents the base of the Wyldcard deck
```
let plinth = new Plinth('devkit')
```

The devkit plinth consists for four _wells_ which are the spots you can place cards. The number of wells and their relative positions could vary, depending on a specific game design, but the devkit has a default of four wells, which are numbered `0` to `3`, from left to right.

Each well has three buttons. This is also an arbitrary decision made for the devkit and could vary depending on the setup designed for a particular game. The buttons are labeled top to bottom `a`, `b` and `c`.

<img width="920" alt="Wyldcard Well Layout" src="https://github.com/Jonahss/wyldcard-public/assets/1521841/d08097f3-ba3b-41b1-a863-bbbb0c1b4324">

Buttons can be referred to by their well and letter.

The `plinth` object has and array of four wells
```
plinth.wells[0]
plinth.wells[1]
plinth.wells[2]
plinth.wells[3]
```

## Display an image on a Wyldcard screen

We need the image we want to display, so this time import the `imageUtilities` as well:

```
let { Plinth, imageUtilities } = require('@wyldcard/drivers')

let plinth = new Plinth('devkit')

let image = await imageUtilities.loadPng('~/Pictures/Wyldcard/CollectionA/Peacock.png')

plinth.wells[0].displayImage(image)
```

We load the image, and call `displayImage(image)` on the corresponding Well.

Images loaded in this manner must be in the proper format. See the [formatting images](docs/formatting-images.md) documentation. Your Wyldcard ships with some images already stored in `/home/pi/Pictures`.

You can also send your own data to the `displayImage(image)` function, make it a Buffer 9472 bytes long where each byte contains four two-bit pairs specifying the color of each pixel where `0b00` is white and `0b11` is black while the two in between are shades of grey.

See the [simple-demo example](examples/simple-demo/index.js) for a full working implementation.


## Respond to Button Presses

You can respond to button presses using a callback function attached to the well, a stream of events emitted by each well, or a stream of "chorded button press" events, which does some logic for you to treat combinations of buttons pressed as a single event.

### Using callbacks

Assign any function you want to the callbacks on each well. It's called once per button press.
```
let well = plinth.wells[0]

let doWhatever = () => console.log('hi')

well.onAButtonPress(doWhatever)
well.onBButtonPress(doWhatever)
well.onCButtonPress(doWhatever)
```

See the [simple-demo example](examples/simple-demo/index.js) for a full working implementation.

### Using events

Each well emits a stream of `'buttonPress'` events which look like:
```
{
  well: 0|1|2|3,
  button: a|b|c,
  ts: Date.now()
}
```
So,
```
let doSomething = (event) => console.log(`pressed button ${event.button} of well ${event.well}`)

plinth.wells.forEach((well) => {
    well.on('buttonPress', doSomething)
})
```

### Chorded button presses with events

Wells also emit `chordedButtonPressEvents` which look like:
```
{
  well: 0|1|2|3,
  buttons: [a|b|c],
  ts: Date.now()
}
```
So, each event has a `buttons` field instead of `button` and it contains an array of strings, representing the buttons pressed on this well.

See the [memory example](examples/memory/index.js) for a full working implementation.

Currently, chording is only supported for a single well, not hitting multiple buttons on different wells at the same time, but this could easily be implemented.


## Read and Write to Memory

Each Wyldcard contains a 4KB memory chip. You can read and write raw bits to it, or conveniently store small objects as json.

To write an object to the memory as JSON:
```
let data = { id: 'abc', name: 'chuckwudi' }
let well = plinth.wells[0]
well.storeData(data)
```

To automatically read the memory and parse it as JSON into an object:
```
let well = plinth.wells[0]
data = well.getData()

console.log(data) // { id: 'abc', name: 'chuckwudi' }
```


You can use `well._writeMemory(buffer)` and `well._readMemory()` to read and write raw bytes to the memory, which will let you use the space more efficiently.

See the [memory example](examples/memory/index.js) for a full working implementation.
