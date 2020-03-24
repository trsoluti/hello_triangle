//! A light-weight wrapper for NSApplicationMain


use std::env::Args;
use std::os::raw::{c_char, c_int};
use std::ptr::null;

#[link(name="AppKit", kind="framework")]
extern {
    // System/Library/Frameworks/AppKit.framework/Versions/C/Headers/NSApplication.h:
    // int NSApplicationMain(int argc, const char *_Nonnull argv[_Nonnull]);
    /// Called by the main function to create and run the application.
    pub fn NSApplicationMain(argc: c_int, argv: *const *const c_char) -> c_int;
}

/// Passes control to Cocoa's NSApplicationMain
/// arguments are currently ignored (just as in NSApplicationMain).
pub fn application_main(_args: Args) {

    // Pass control to NSApplicationMain
    // args are ignored according to the doc.
    unsafe { NSApplicationMain(0, null()) };
}