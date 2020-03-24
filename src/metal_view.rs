//! A version of NSView that draws using Metal
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use objc::class;
use objc::sel;
use objc::sel_impl;
use objc::msg_send;
use std::os::raw::c_double;
use objc::{Encode, Encoding};
use crate::renderer::Renderer;
use cocoa::base::{id, nil};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel, BOOL, objc_retain, objc_release};
use crate::display_link::{DisplayLink, dispatch_queue_t};
use cocoa::foundation::{NSUInteger, NSRect, NSSize, NSAutoreleasePool};
use std::ffi::c_void;

// From Metal.framework/Versions/A/Headers/MTLRenderPass.h
// in XCode MacOS.sdk:
// typedef struct
// {
//     double red;
//     double green;
//     double blue;
//     double alpha;
// } MTLClearColor;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MTLClearColor {
    red: c_double,
    green: c_double,
    blue: c_double,
    alpha: c_double,
}
unsafe impl Encode for MTLClearColor {
    fn encode() -> Encoding {
        let encoding = format!("{{MTLClearColor={}{}{}{}}}",
                               c_double::encode().as_str(),
                               c_double::encode().as_str(),
                               c_double::encode().as_str(),
                               c_double::encode().as_str());
        unsafe { objc::Encoding::from_str(&encoding) }
    }
}
// MTL_INLINE MTLClearColor MTLClearColorMake(double red, double green, double blue, double alpha);
#[allow(non_snake_case)]
pub fn MTLClearColorMake(red: c_double, green: c_double, blue: c_double, alpha: c_double) -> MTLClearColor {
    MTLClearColor {red, green, blue, alpha }
}

// From System/Library/Frameworks/CoreGraphics.framework/Versions/A/Headers/CGGeometry.h:
// struct CGSize {
//     CGFloat width;
//     CGFloat height;
// };
// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct CGSize {
//     pub width: CGFloat,
//     pub height: CGFloat,
// }
// unsafe impl Encode for CGSize {
//     fn encode() -> Encoding {
//         let encoding = format!("{{CGSize={}{}}}",
//                                CGFloat::encode().as_str(),
//                                CGFloat::encode().as_str());
//         unsafe { objc::Encoding::from_str(&encoding) }
//     }
// }
pub type CGSize = NSSize; // technically, it's the other way around

// From System/Library/Frameworks/Metal.framework/Versions/A/Headers/MTLPixelFormat.h:
// typedef NS_ENUM(NSUInteger, MTLPixelFormat) {...}
pub type MTLPixelFormat = NSUInteger;
// MTLPixelFormatBGRA8Unorm      = 80
static MTLPixelFormatBGRA8Unorm:MTLPixelFormat = 80;

#[link(name="GlueLib", kind="dylib")]
extern {
    fn dispatch_get_main_queue_not_inline() -> dispatch_queue_t;
}

// From MTLRenderPass.h:
// typedef NS_ENUM(NSUInteger, MTLLoadAction) {
//     MTLLoadActionDontCare = 0,
//     MTLLoadActionLoad = 1,
//     MTLLoadActionClear = 2,
// } API_AVAILABLE(macos(10.11), ios(8.0));
static MTLLoadActionClear: NSUInteger = 2;

pub trait MetalViewDelegate: Sized {
    fn metal_view_drawable_size_will_change(&mut self, size: CGSize);
    fn draw_in_metal_view(&self);
}

/// The Rust portion of the class that handles the view
pub struct RSMetalView {
    timer: Box<DisplayLink>,
    clear_color: MTLClearColor,
    delegate: Option<Box<Renderer>>,
    device: id,
    enable_set_needs_display: bool,
    current_render_pass_descriptor: id,
    current_drawable: id,
    drawable_size: CGSize,
}

impl RSMetalView {
    fn set_current_render_pass_descriptor(&mut self, render_descriptor: id) {
        if self.current_render_pass_descriptor != nil {
            unsafe { objc_release(self.current_render_pass_descriptor) };
            self.current_render_pass_descriptor = nil
        }
        if render_descriptor != nil {
            self.current_render_pass_descriptor = unsafe { objc_retain(render_descriptor) };
        }
    }
    fn set_current_drawable(&mut self, current_drawable: id) {
        if self.current_drawable != nil {
            unsafe { objc_release(self.current_drawable) };
            self.current_drawable = nil
        }
        if current_drawable != nil {
            self.current_drawable = unsafe { objc_retain(current_drawable) };
        }
    }
}

pub fn register_metal_view_class() {
    let ns_view_class = class!(NSView);
    let mut metal_view_declaration = ClassDecl::new("MetalView", ns_view_class).unwrap();
    unsafe {
        metal_view_declaration.add_ivar::<*mut c_void>("_rustMetalView");
        metal_view_declaration.add_method(
            sel!(initWithCoder:),
            init_with_coder_ as extern "C" fn(&Object, Sel, id) -> id,
        );
        metal_view_declaration.add_method(
            sel!(device),
            get_device as extern "C" fn(&Object, Sel) -> id,
        );
        metal_view_declaration.add_method(
            sel!(setDevice:),
            set_device_ as extern "C" fn(&mut Object, Sel, id),
        );
        metal_view_declaration.add_method(
            sel!(enableSetNeedsDisplay),
            get_enable_set_needs_display as extern "C" fn(&Object, Sel) -> BOOL,
        );
        metal_view_declaration.add_method(
            sel!(setEnableSetNeedsDisplay:),
            set_enable_set_needs_display_ as extern "C" fn(&mut Object, Sel, BOOL)
        );
        metal_view_declaration.add_method(
            sel!(delegate),
            get_delegate as extern "C" fn(&Object, Sel) -> id,
        );
        metal_view_declaration.add_method(
            sel!(setDelegate:),
            // set_delegate_ as extern "C" fn(&mut Object, Sel, id),
            set_delegate_ as extern "C" fn(&mut Object, Sel, *mut c_void),
        );
        metal_view_declaration.add_method(
            sel!(colorPixelFormat),
            get_color_pixel_format as extern "C" fn(&Object, Sel) -> MTLPixelFormat,
        );
        metal_view_declaration.add_method(
            sel!(drawableSize),
            get_drawable_size as extern "C" fn(&Object, Sel) -> CGSize,
        );
        metal_view_declaration.add_method(
            sel!(setDrawableSize:),
            set_drawable_size_ as extern "C" fn(&mut Object, Sel, CGSize),
        );
        metal_view_declaration.add_method(
            sel!(makeBackingLayer),
            make_backing_layer as extern "C" fn(&Object, Sel) -> id,
        );
        metal_view_declaration.add_method(
            sel!(currentRenderPassDescriptor),
            get_current_render_pass_descriptor as extern "C" fn(&Object, Sel) -> id,
        );
        metal_view_declaration.add_method(
            sel!(currentDrawable),
            get_current_drawable as extern "C" fn(&Object, Sel) -> id,
        );
        metal_view_declaration.add_method(
            sel!(setClearColor:),
            set_clear_color as extern "C" fn(&mut Object, Sel, MTLClearColor),
        );
    }
    metal_view_declaration.register();
}

extern "C" fn init_with_coder_(_self: &Object, _sel: Sel, _coder: id) -> id {
    let _self: id = unsafe {
        let _superclass = class!(NSView);
        msg_send![super(_self, _superclass), initWithCoder: _coder]
    };
    println!("In MetalView::init_with_coder");

    if _self != nil {
        let pool = unsafe { NSAutoreleasePool::new(nil) };
        let enable_set_needs_display = false;
        let clear_color = MTLClearColorMake(0., 0., 0., 1.);

        let timer = unsafe {
            let dispatch_main_queue = dispatch_get_main_queue_not_inline();
            let timer_result = DisplayLink::new_with_queue_and_callback(
                dispatch_main_queue,
                timer_callback,
                _self,
            );
            if let Err(e) = timer_result {
                println!("Display link error {:?}", e);
                return nil;
            };
            timer_result.unwrap()
        };
        let frame: NSRect = unsafe { msg_send![_self, frame] };
        let _size = frame.size;
        let drawable_size:NSSize = unsafe { msg_send![_self, convertSizeToBacking:_size] };

        let _:() = unsafe { msg_send![_self, setWantsLayer:true] };

        let _rust_metal_view = Box::new( RSMetalView {
            timer,
            clear_color,
            delegate: None,
            device: nil,
            enable_set_needs_display,
            current_render_pass_descriptor: nil,
            current_drawable: nil,
            drawable_size,
        });
        let _raw_ptr = Box::into_raw(_rust_metal_view) as *mut c_void;
        //let _:() = unsafe { msg_send![_self, setRustMetalView:_raw_ptr] };
        unsafe { _self.as_mut().unwrap().set_ivar("_rustMetalView", _raw_ptr)}
        unsafe { pool.drain() }
    }
    _self
}

extern "C" fn get_device(_self: &Object, _sel: Sel) -> id {
    get_rust_metal_view(_self).device
}
extern "C" fn set_device_(_self: &mut Object, _sel: Sel, new_device: id) {
    get_mut_rust_metal_view(_self).device = new_device;
    if new_device != nil {
        let metal_layer: id = unsafe { msg_send![_self, layer] };
        if metal_layer != nil {
            let _:() = unsafe { msg_send![metal_layer, setDevice:new_device] };
        }
    }
}

extern "C" fn get_enable_set_needs_display(_self: &Object, _sel: Sel) -> BOOL {
    get_rust_metal_view(_self).enable_set_needs_display as BOOL
}
extern "C" fn set_enable_set_needs_display_(_self: &mut Object, _sel: Sel, new_value: BOOL) {
    get_mut_rust_metal_view(_self).enable_set_needs_display = new_value != 0
}

extern "C" fn get_delegate(_self: &Object, _sel: Sel) -> id {
    assert!(false); // if we call this, then something is wrong.
    nil
}
extern "C" fn set_delegate_(_self: &mut Object, _sel: Sel, new_delegate: *mut c_void) {
    let mut rust_metal_view = get_mut_rust_metal_view(_self);
    let new_delegate_ref = unsafe { new_delegate.as_ref() };
    let new_renderer = match new_delegate_ref {
        Some(_) => {
            let new_renderer_ref = new_delegate as *mut Renderer;
            let new_renderer_box = unsafe { Box::from_raw(new_renderer_ref) };
            Some(new_renderer_box)
        },
        None => None
    };

    if new_renderer.is_some() {
        rust_metal_view.timer.start()
    } else {
        rust_metal_view.timer.stop()
    }

    rust_metal_view.delegate = new_renderer;
}
extern "C" fn make_backing_layer(_self: &Object, _sel: Sel) -> id {
    let pool = unsafe { NSAutoreleasePool::new(nil) };
    #[allow(unused)]
    let ca_metal_layer_class = unsafe { class!(CAMetalLayer) };
    let layer:id = unsafe { msg_send![ca_metal_layer_class, layer] };
    #[allow(unused)]
    let square = CGSize { width: 1., height: 1. };
    let view_scale: CGSize = unsafe { msg_send![_self, convertSizeToBacking:square] };
    #[allow(unused)]
    let contents_scale = f64::min(view_scale.width, view_scale.height);
    let _:() = unsafe { msg_send![layer, setContentsScale:contents_scale] };
    let layer = unsafe { objc_retain(layer) };
    unsafe { pool.drain() };
    layer
}

extern "C" fn get_color_pixel_format(_self: &Object, _sel: Sel) -> MTLPixelFormat {
    if let Some(metal_layer) = get_metal_layer(_self) {
        unsafe { msg_send![metal_layer, pixelFormat] }
    } else {
        MTLPixelFormatBGRA8Unorm
    }
}

extern "C" fn get_drawable_size(_self: &Object, _sel: Sel) -> CGSize {
    get_rust_metal_view(_self).drawable_size
}
extern "C" fn set_drawable_size_(_self: &mut Object, _sel: Sel, new_drawable_size: CGSize) {
    let _drawable_size = get_rust_metal_view(_self).drawable_size;

    if new_drawable_size.height != _drawable_size.height
        || new_drawable_size.width != _drawable_size.width {
        if let Some(metal_layer) = get_metal_layer(_self) {
            let _:() = unsafe { msg_send![metal_layer, setDrawableSize:new_drawable_size] };
        }
        // let layer: id = unsafe { msg_send![_self, layer] };
        // if layer != nil {
        //     let _:() = unsafe { msg_send![layer, setDrawableSize:new_drawable_size] };
        // }

        if let Some(delegate) = get_mut_rust_metal_view(_self).delegate.as_mut() {
            delegate.metal_view_drawable_size_will_change(new_drawable_size);
        }
    }

    get_mut_rust_metal_view(_self).drawable_size = new_drawable_size
}
extern "C" fn get_current_render_pass_descriptor(_self: &Object, _sel: Sel) -> id {
    get_rust_metal_view(_self).current_render_pass_descriptor
}
extern "C" fn get_current_drawable(_self: &Object, _sel: Sel) -> id {
    get_rust_metal_view(_self).current_drawable
}

extern "C" fn set_clear_color(_self: &mut Object, _sel: Sel, _color: MTLClearColor) {
    get_mut_rust_metal_view(_self).clear_color = _color
}


fn get_metal_layer(_self: &Object) -> Option<&Object> {
    let metal_layer:id = unsafe { msg_send![_self, layer] };
    unsafe { metal_layer.as_ref() }
}

fn get_rust_metal_view(_self: &Object) -> &RSMetalView {
    unsafe {
        let mut _raw_ptr: *mut c_void = *_self.get_ivar("_rustMetalView");
        (_raw_ptr as *const RSMetalView).as_ref().unwrap()
    }
}

fn get_mut_rust_metal_view(_self: &mut Object) -> &mut RSMetalView {
    unsafe {
        let mut _raw_ptr: *mut c_void = *_self.get_mut_ivar("_rustMetalView");
        (_raw_ptr as *mut RSMetalView).as_mut().unwrap()
    }
}

fn timer_callback(_self: &mut Object) {
    let frame: NSRect = unsafe { msg_send![_self, frame] };
    let _size = frame.size;
    let backing_size:NSSize = unsafe { msg_send![_self, convertSizeToBacking:_size] };
    // get_mut_rust_metal_view(_self).drawable_size = backing_size;
    let _:() = unsafe { msg_send![_self, setDrawableSize:backing_size]};

    set_up_delegate_drawing_state(_self);

    if let Some(renderer) = get_rust_metal_view(_self).delegate.as_ref() {
        renderer.draw_in_metal_view()
    }
}

fn set_up_delegate_drawing_state(_self: &mut Object) {
    // Set up our render pass descriptor
    let render_descriptor_class = class!(MTLRenderPassDescriptor);
    let render_descriptor: id = unsafe { msg_send![render_descriptor_class, alloc] };
    let render_descriptor: id = unsafe { msg_send![render_descriptor, init] };
    get_mut_rust_metal_view(_self).set_current_render_pass_descriptor(render_descriptor);

    // Set up our drawable
    if let Some(metal_layer) = get_metal_layer(_self) {
        let current_drawable: id = unsafe { msg_send![metal_layer, nextDrawable] };

        get_mut_rust_metal_view(_self).set_current_drawable(current_drawable);
    }

    let current_drawable = get_rust_metal_view(_self).current_drawable;
    if let Some(current_render_pass_descriptor) = unsafe { render_descriptor.as_mut() } {
        unsafe {
            let _render_pass_color_attachment_descriptor_array: id =
                msg_send![current_render_pass_descriptor, colorAttachments];
            let color_attachment_0: id =
                msg_send![_render_pass_color_attachment_descriptor_array, objectAtIndexedSubscript:0];

            if color_attachment_0 != nil {
                if current_drawable != nil {
                    let _texture: id = msg_send![current_drawable, texture];
                    let _:() = msg_send![color_attachment_0, setTexture:_texture];
                }
                let _:() = msg_send![color_attachment_0, setLoadAction:MTLLoadActionClear];

                let clear_color = get_rust_metal_view(_self).clear_color;
                let _:() = msg_send![color_attachment_0, setClearColor:clear_color];
            }
            let _:() = msg_send![_render_pass_color_attachment_descriptor_array,
                                 setObject:color_attachment_0 atIndexedSubscript:0];
        }
    }
}
