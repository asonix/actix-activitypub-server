#![feature(associated_type_defaults)]

extern crate actix;
extern crate actix_web;
extern crate futures;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate tokio_timer;

pub mod actors;
