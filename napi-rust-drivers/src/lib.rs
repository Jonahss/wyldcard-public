#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode},
  JsBoolean, JsString,
};
use rppal::gpio::Level;

mod plinth;
mod epaper_display;

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
    self.plinth.display_image(well.into(), buf);
  }

  #[napi]
  pub fn set_switch_callback(&mut self, well: u8, switch: String, callback: JsFunction) -> Result<()> {
    let s:char = match switch.as_str() {
      "a" => 'a',
      "b" => 'b',
      "c" => 'c',
      _ => return Err(Error::new(Status::InvalidArg, "Must supply switch argument as a string 'a', 'b', or 'c'"))
    };

    let tsfn: ThreadsafeFunction<u32, ErrorStrategy::CalleeHandled> = callback.create_threadsafe_function(0, |ctx| {
      Ok(vec![ctx.value + 1])
    })?;

    let switch_callback = move |level| {
      tsfn.call(Ok(1), ThreadsafeFunctionCallMode::NonBlocking);
    };

    match self.plinth.set_switch_callback(well.into(), s, switch_callback) {
      Ok(_) => Ok(()),
      Err(e) => Err(Error::new(Status::GenericFailure, e))
    }
  }
}
