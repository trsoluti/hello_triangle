//! A Renderer to draw in our view

use objc::class;
use objc::msg_send;
use objc::sel;
use objc::sel_impl;
use cocoa::base::{id, nil};
use crate::metal_view::{CGSize, MetalViewDelegate, MTLPixelFormat};
use std::fmt::Formatter;
use std::error::Error;
use crate::vector_types::{vector_uint2, vector_float2, vector_float4};
use cocoa::foundation::{NSAutoreleasePool, NSString, NSUInteger};
use objc::runtime::objc_retain;
use std::os::raw::{c_uint, c_double};

// From System/Library/Frameworks/Metal.framework/Versions/A/Headers/MTLRenderCommandEncoder.h
// typedef struct {
//     double originX, originY, width, height, znear, zfar;
// } MTLViewport;
#[repr(C)]
struct MTLViewport {
    origin_x: c_double,
    origin_y: c_double,
    width:    c_double,
    height:   c_double,
    z_near:   c_double,
    z_far:    c_double,
}

// From System/Library/Frameworks/Metal.framework/Versions/A/Headers/MTLRenderCommandEncoder.h
#[allow(non_upper_case_globals)]
static MTLPrimitiveTypeTriangle:NSUInteger = 3;


#[derive(Debug)]
pub enum RendererInitError {
    FailedToInit,
    UnableToSetPipelineState,
}
impl std::fmt::Display for RendererInitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToInit => write!(f, "Failed to initialize"),
            Self::UnableToSetPipelineState => write!(f, "Unable to set pipeline state"),
        }
    }
}
impl Error for RendererInitError{}


/// Renderer to draw in our view
pub struct Renderer {
    view: id,
    pipeline_state: id,
    command_queue: id,
    viewport_size: vector_uint2,
}

impl Renderer {
    /// Creates a new renderer with the given view
    pub fn new_with_metal_kit_view(view: id) -> Result<Self, RendererInitError> {
        let pool = unsafe { NSAutoreleasePool::new(nil) };
        let device: id = unsafe { msg_send![view, device] };
        let default_library: id = unsafe { msg_send![device, newDefaultLibrary] };
        let vertex_shader_name = unsafe { NSString::alloc(nil).init_str("vertexShader") };
        let vertex_function: id = unsafe { msg_send![default_library, newFunctionWithName:vertex_shader_name] };
        let fragment_function_name = unsafe { NSString::alloc(nil).init_str("fragmentShader") };
        let fragment_function: id = unsafe { msg_send![default_library, newFunctionWithName:fragment_function_name] };

        // Configure a pipeline descriptor that is used to create a pipeline state.
        let render_pipeline_descriptor_class = class!(MTLRenderPipelineDescriptor);
        let pipeline_state_descriptor: id = unsafe { msg_send![render_pipeline_descriptor_class, alloc] };
        let pipeline_state_descriptor: id = unsafe { msg_send![pipeline_state_descriptor, init] };

        let pipeline_label = unsafe { NSString::alloc(nil).init_str("Simple Pipeline") };
        let _:() = unsafe {msg_send![pipeline_state_descriptor, setLabel:pipeline_label] };

        let _:() = unsafe { msg_send![pipeline_state_descriptor, setVertexFunction:vertex_function] };
        let _:() = unsafe { msg_send![pipeline_state_descriptor, setFragmentFunction:fragment_function] };

        let color_attachment_array: id = unsafe { msg_send![pipeline_state_descriptor, colorAttachments] };
        let color_attachment_0: id = unsafe { msg_send![color_attachment_array, objectAtIndexedSubscript:0] };
        let pixel_format: MTLPixelFormat = unsafe { msg_send![view, colorPixelFormat] };
        let _:() = unsafe { msg_send![color_attachment_0, setPixelFormat:pixel_format] };
        let _:() = unsafe { msg_send![color_attachment_array, setObject:color_attachment_0 atIndexedSubscript:0] };

        let error: id = nil;
        let pipeline_state: id = unsafe { msg_send![device, newRenderPipelineStateWithDescriptor:pipeline_state_descriptor error: &error] };
        if pipeline_state == nil {
            unsafe { pool.drain() };
            return Err(RendererInitError::UnableToSetPipelineState)
        }

        let command_queue: id = unsafe { msg_send![device, newCommandQueue] };

        let pipeline_state = unsafe { objc_retain(pipeline_state) };
        let command_queue = unsafe { objc_retain(command_queue) };
        unsafe { pool.drain() };
        Ok( Renderer {
            view,
            pipeline_state,
            command_queue,
            viewport_size: vector_uint2::new(0, 0), // will be set by view immediately
        })
    }
}

impl MetalViewDelegate for Renderer {
    fn metal_view_drawable_size_will_change(&mut self, _size: CGSize) {
        let width: u32 = _size.width as u32;
        let height: u32 = _size.height as u32;
        let new_viewport_size = vector_uint2::new(
            width,
            height,
        );
        //+ println!("Setting viewport size to ({},{}), ({},{}) {}", _size.width, _size.height, new_viewport_size.x(), new_viewport_size.y(), new_viewport_size);
        // unsafe { _self.set_ivar("_viewportSize", new_viewport_size) };
        self.viewport_size = new_viewport_size;
    }

    fn draw_in_metal_view(&self) {
        //+ println!("In draw in metal view");
        let pool = unsafe { NSAutoreleasePool::new(nil) };

        // Create a new command buffer for each render pass to the current drawable.
        let command_queue: id = self.command_queue;
        let command_buffer: id = unsafe { msg_send![command_queue, commandBuffer] };
        let label_name = unsafe { NSString::alloc(nil).init_str("MyCommand") };
        let _:() = unsafe { msg_send![command_buffer, setLabel:label_name] };

        // Obtain a renderPassDescriptor generated from the view's drawable textures.
        let render_pass_descriptor: id = unsafe { msg_send![self.view, currentRenderPassDescriptor] };

        if render_pass_descriptor != nil {
            let render_encoder:id = unsafe {
                msg_send![command_buffer,
                          renderCommandEncoderWithDescriptor:render_pass_descriptor]
            };
            let render_encoder_name = unsafe { NSString::alloc(nil).init_str("MyRenderEncoder") };
            let _:() = unsafe {msg_send![render_encoder, setLabel:render_encoder_name] };

            let viewport_size: vector_uint2 = self.viewport_size;
            //+ println!("Width and height of viewport are ({},{})", viewport_size.x(), viewport_size.y());
            let viewport = MTLViewport {
                origin_x: 0.0,
                origin_y: 0.0,
                width: f64::from(viewport_size.x()),
                height: f64::from(viewport_size.y()),
                z_near: 0.0,
                z_far: 1.0
            };
            let _:() = unsafe { msg_send![render_encoder, setViewport:viewport] };

            let pipeline_state: id = self.pipeline_state;
            let _:() = unsafe { msg_send![render_encoder, setRenderPipelineState:pipeline_state] };

            let triangle_vertices = AAPLVertices::default();
            let _tv_size = std::mem::size_of_val(&triangle_vertices);
            //+ println!("size of triangle vertices is {}", _tv_size);
            let _:() = unsafe { msg_send![render_encoder, setVertexBytes:&triangle_vertices length:_tv_size atIndex:AAPLVertexInputIndexVertices] };

            let _viewport_size_size = std::mem::size_of_val(&viewport_size);
            //+ println!("size of viewport_size is {}", _viewport_size_size);
            let _:() = unsafe { msg_send![render_encoder, setVertexBytes:&viewport_size length:_viewport_size_size atIndex:AAPLVertexInputIndexViewportSize] };

            let _:() = unsafe { msg_send![render_encoder, drawPrimitives:MTLPrimitiveTypeTriangle vertexStart:0 vertexCount:3] };

            let _:() = unsafe { msg_send![render_encoder, endEncoding] };

            let _current_drawable: id = unsafe { msg_send![self.view, currentDrawable] };
            let _:() = unsafe { msg_send![command_buffer, presentDrawable:_current_drawable] };
        }

        let _:() = unsafe { msg_send![command_buffer, commit] };
        unsafe { pool.drain() };
    }
}

// From AAPLShaderTypes.h:
// Buffer index values shared between shader and C code to ensure Metal shader buffer inputs
// match Metal API buffer set calls.
// typedef enum AAPLVertexInputIndex
// {
//   AAPLVertexInputIndexVertices     = 0,
//   AAPLVertexInputIndexViewportSize = 1,
// } AAPLVertexInputIndex;
#[allow(non_upper_case_globals)]
static AAPLVertexInputIndexVertices: c_uint     = 0;
#[allow(non_upper_case_globals)]
static AAPLVertexInputIndexViewportSize: c_uint = 1;
//
//  This structure defines the layout of vertices sent to the vertex
//  shader. This header is shared between the .metal shader and C code, to guarantee that
//  the layout of the vertex array in the C code matches the layout that the .metal
//  vertex shader expects.
// typedef struct
// {
//   vector_float2 position;
//   vector_float4 color;
// } AAPLVertex;
#[repr(C, packed(4))]
struct AAPLVertex {
    position: vector_float2,
    color: vector_float4,
}
//
// static const AAPLVertex triangleVertices[] =
// {
//     // 2D positions,    RGBA colors
//     { {  250,  -250 }, { 1, 0, 0, 1 } },
//     { { -250,  -250 }, { 0, 1, 0, 1 } },
//     { {    0,   250 }, { 0, 0, 1, 1 } },
// };
struct AAPLVertices {
    _vertices: [AAPLVertex; 3],
}
impl Default for AAPLVertices {
    fn default() -> Self {
        AAPLVertices {
            _vertices: [
                AAPLVertex {
                    position: vector_float2::new(250., -250.),
                    color: vector_float4::new(1., 0., 0., 1.),
                },
                AAPLVertex {
                    position: vector_float2::new(-250., -250.),
                    color: vector_float4::new(0., 1., 0., 1.),
                },
                AAPLVertex {
                    position: vector_float2::new(0., 250.),
                    color: vector_float4::new(0., 0., 1., 1.),
                },
            ]
        }
    }
}
