#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod plinth;
mod epaper_display;

use napi::bindgen_prelude::Buffer;

use crate::plinth::{ Plinth, DevKitV1, Prototype };

#[napi]
struct JsPrototype {
  plinth: Prototype,
}

#[napi]
impl JsPrototype {
  #[napi(constructor)]
  pub fn new() -> Self {
    JsPrototype { plinth: Prototype::new() }
  }

  #[napi]
  pub fn display_image(&self, well: u8, image: Buffer) {
    let buf: Vec<u8> = image.into();
    self.plinth.display_image(well.into(), buf)
  }
}


#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
