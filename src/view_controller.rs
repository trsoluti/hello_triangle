//! Handle the view controller class and callbacks

use objc::class;
use objc::sel;
use objc::sel_impl;
use objc::msg_send;
use cocoa::base::{id, nil};
use objc::runtime::{Object, Sel};
use crate::renderer::Renderer;
use objc::declare::ClassDecl;
use std::ffi::c_void;
use cocoa::foundation::NSAutoreleasePool;
use crate::metal_view::{MTLClearColorMake, CGSize, MetalViewDelegate};

#[link(name="Metal", kind="framework")]
extern {
    // From System/Library/Frameworks/Metal.framework/Versions/A/Headers/MTLDevice.h:
    // MTL_EXTERN id <MTLDevice> __nullable MTLCreateSystemDefaultDevice(void) API_AVAILABLE(macos(10.11), ios(8.0)) NS_RETURNS_RETAINED;
    fn MTLCreateSystemDefaultDevice() -> id;
}

/// The Rust companion to the Objc ViewController class
pub struct RSViewController {
    /// The render that will draw in our main view.
    _renderer: Option<Box<Renderer>>,
}

/// Register the ViewController class, iVars and callbacks
pub fn register_view_controller_class() {
    let _ns_view_controller_class = class!(NSViewController);
    let mut view_controller_declaration = ClassDecl::new("ViewController", _ns_view_controller_class).unwrap();
    unsafe {
        view_controller_declaration.add_ivar::<*mut c_void>("_rust_instance_ptr");
        view_controller_declaration.add_method(
            sel!(initWithCoder:),
            init_with_coder_ as extern "C" fn(&Object, Sel, id) -> id,
        );
        view_controller_declaration.add_method(
            sel!(viewDidLoad),
            view_did_load as extern "C" fn(&mut Object, Sel),
        )
    }
    view_controller_declaration.register();
}

/// The designated initializer for NSViewController.
///
/// We create a RSViewController instance and store it as an iVar.
extern "C" fn init_with_coder_(_self: &Object, _sel: Sel, _coder: id) -> id {
    // init our parent class from the coder
    let mut _self:id = unsafe {
        let _superclass = class!(NSViewController);
        msg_send!(super(_self, _superclass), initWithCoder:_coder)
    };
    println!("In ViewController init with coder!");

    // Add in a new, boxed RSViewController instance as an iVar
    let _rust_instance = Box::new(RSViewController {
        _renderer: None,
    });
    println!("  _rust_instance_ptr is {:p}", _rust_instance);
    let _rust_instance_ptr = Box::into_raw(_rust_instance) as *mut c_void;
    unsafe { _self.as_mut().unwrap().set_ivar("_rust_instance_ptr", _rust_instance_ptr) };
    _self
}

/// Called after the view controller’s view has been loaded into memory.
extern "C" fn view_did_load(_self: &mut Object, _sel: Sel) {
    unsafe {
        let _superclass = class!(NSViewController);
        let _: () = msg_send!(super(_self, _superclass), viewDidLoad);
    }
    println!("In view did load!");


    // Do any additional setup after loading the view.
    //
    unsafe {
        let pool = NSAutoreleasePool::new(nil);

        // Recover our rust instance
        let mut _rust_instance_ptr: *mut c_void = *_self.get_mut_ivar("_rust_instance_ptr");
        println!("  _rust_instance_ptr set to {:?}", _rust_instance_ptr);
        let mut _rust_instance_ptr = (_rust_instance_ptr as *mut RSViewController).as_mut().unwrap();

        // Set up our view
        let view: id = msg_send![_self, view];
        let _: () = msg_send![view, setEnableSetNeedsDisplay:true];

        let new_device = MTLCreateSystemDefaultDevice();
        let _: () = msg_send![view, setDevice:new_device];

        let clear_color = MTLClearColorMake(0.0, 0.5, 1.0, 1.0);
        let _: () = msg_send![view, setClearColor:clear_color];

        // Create a renderer for our view
        let renderer_result = Renderer::new_with_metal_kit_view(view);
        match renderer_result {
            Ok(renderer) => {
                _rust_instance_ptr._renderer = Some(Box::new(renderer));
            }
            _ => {
                println!("Renderer initialization failed");
                pool.drain();
                return;
            }
        }

        // Initialize the renderer with the view size.
        let drawable_size: CGSize = msg_send![view, drawableSize];
        _rust_instance_ptr._renderer.as_mut().unwrap().metal_view_drawable_size_will_change(drawable_size);

        // pass our renderer to the view as a pointer
        // note that the view can't think of it as a Objc delegate
        let _renderer = _rust_instance_ptr._renderer.as_ref().unwrap().as_ref();
        let _: () = msg_send![view, setDelegate:_renderer];
        pool.drain();
    }
}