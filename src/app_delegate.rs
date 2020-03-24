//! Handle the App Delegate class and callbacks
//!
//! Note the registered name **has** to be the same
//! as the name in the storyboard.

use objc::class;
use objc::sel;
use objc::sel_impl;
use objc::runtime::{Object, Sel};
use objc::declare::ClassDecl;
use cocoa::base::id;

/// Register the AppDelegate class and callbacks
/// with the Objc Runtime.
pub fn register_app_delegate_class() {
    let ns_object_class = class!(NSObject);
    let mut app_delegate_builder = ClassDecl::new("AppDelegate", ns_object_class).unwrap();

    // Add the two callbacks in which we are interested:
    unsafe {
        app_delegate_builder.add_method(
        sel!(applicationDidFinishLaunching:),
        application_did_finish_launching as extern "C" fn(&Object, Sel, id),
        );
        app_delegate_builder.add_method(
            sel!(applicationWillTerminate:),
            application_will_terminate as extern "C" fn(&Object, Sel, id),
        );
    };

    app_delegate_builder.register();
}

/// This function is called by the run loop
/// when the application has finished launching
extern "C" fn application_did_finish_launching(_self: &Object, _sel: Sel, _a_notification: id) {
    // Insert code here to initialize your application
    println!("In application did finish launching!");
}

/// This function is called by the run loop
/// when the application is about to shut down
extern "C" fn application_will_terminate(_self: &Object, _sel: Sel, _a_notification: id) {
    // Insert code here to initialize your application
    println!("In application will terminate!");
}
