#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod plinth;
mod epaper_display;

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
  a + b
}
