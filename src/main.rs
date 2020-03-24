//! A simple application showing a Rust program passing control
//! to the Cocoa runtime system and interacting with it
//! to write directly to the Metal driver.
//!
//! The general flow of control is:
//!
//! Main registers our Objective C classes
//! and then passes control to NSApplicationMain
//! NSApplicationMain sets up the threads and reads the storyboard.
//!
//! The process of setting up triggers callbacks
//! into our Objective C classes.
//!
//! We keep a registry ourselves of each boxed Rust instance,
//! so we can switch back and forth between Rust and Objective C
//! class information.

#![deny(missing_docs)]

// public, so it will get documented
pub use crate::app_delegate::register_app_delegate_class;
use crate::view_controller::register_view_controller_class;
pub use crate::application_main::application_main;
use crate::metal_view::register_metal_view_class;

mod application_main;
mod app_delegate;
mod view_controller;
mod metal_view;
mod display_link;
mod renderer;
mod vector_types; // our kludge of simd "OpenCL Vector Types".

/// Main method
pub fn main() {
    // Register our classes
    // with the Objective C Runtime
    register_app_delegate_class();
    register_view_controller_class();
    register_metal_view_class();

    // Pass control to the NSApplicationMain
    application_main(std::env::args());
}
