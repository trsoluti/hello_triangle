//
//  Renderer.m
//  HelloTriangle
//
//  Created by TR Solutions on 22/3/20.
//  Copyright © 2020 TR Solutions Pte. Ltd.
//  Licensed under Apache 2.0 and MIT
//  See appropriate LICENCE files for details.
//
//  Based on original code provided by Apple.
//  See LICENCE-APPLE file for details.
//

@import simd;

#import "Renderer.h"
#import "AAPLShaderTypes.h"

@implementation Renderer
{
  id<MTLDevice> _device;
  
  // The render pipeline generated from the vertex and fragment shaders in the .metal shader file.
  id<MTLRenderPipelineState> _pipelineState;
  
  // The command queue used to pass commands to the device.
  id<MTLCommandQueue> _commandQueue;
  
  // The current size of the view, used as an input to the vertex shader.
  vector_uint2 _viewportSize;
}

//- (nonnull instancetype)initWithMetalKitView:(nonnull MTKView *)mtkView
- (nonnull instancetype)initWithMetalKitView:(nonnull MetalView *)metalView
{
  self = [super init];
  if(self)
  {
    NSError *error = NULL;
    
    _device = metalView.device;
    
    // Load all the shader files with a .metal file extension in the project.
    id<MTLLibrary> defaultLibrary = [_device newDefaultLibrary];
    
    id<MTLFunction> vertexFunction = [defaultLibrary newFunctionWithName:@"vertexShader"];
    id<MTLFunction> fragmentFunction = [defaultLibrary newFunctionWithName:@"fragmentShader"];
    
    // Configure a pipeline descriptor that is used to create a pipeline state.
    MTLRenderPipelineDescriptor *pipelineStateDescriptor = [[MTLRenderPipelineDescriptor alloc] init];
    pipelineStateDescriptor.label = @"Simple Pipeline";
    pipelineStateDescriptor.vertexFunction = vertexFunction;
    pipelineStateDescriptor.fragmentFunction = fragmentFunction;
    pipelineStateDescriptor.colorAttachments[0].pixelFormat = metalView.colorPixelFormat;
    
    _pipelineState = [_device newRenderPipelineStateWithDescriptor:pipelineStateDescriptor
                                                             error:&error];
    
    // Pipeline State creation could fail if the pipeline descriptor isn't set up properly.
    //  If the Metal API validation is enabled, you can find out more information about what
    //  went wrong.  (Metal API validation is enabled by default when a debug build is run
    //  from Xcode.)
    NSAssert(_pipelineState, @"Failed to created pipeline state: %@", error);
    
    // Create the command queue
    _commandQueue = [_device newCommandQueue];
  }
  
  return self;
}

/// Called whenever view changes orientation or is resized
//- (void)mtkView:(nonnull MTKView *)view drawableSizeWillChange:(CGSize)size
- (void)metalView:(nonnull MetalView *)view drawableSizeWillChange:(CGSize)size
{
  // Save the size of the drawable to pass to the vertex shader.
  _viewportSize.x = size.width;
  _viewportSize.y = size.height;
}

/// Called whenever the view needs to render a frame.
//- (void)drawInMTKView:(nonnull MTKView *)view
- (void)drawInMetalView:(nonnull MetalView *)view
{
  static const AAPLVertex triangleVertices[] =
  {
    // 2D positions,    RGBA colors
    { {  250,  -250 }, { 1, 0, 0, 1 } },
    { { -250,  -250 }, { 0, 1, 0, 1 } },
    { {    0,   250 }, { 0, 0, 1, 1 } },
  };
  
  // Create a new command buffer for each render pass to the current drawable.
  id<MTLCommandBuffer> commandBuffer = [_commandQueue commandBuffer];
  commandBuffer.label = @"MyCommand";
  
  // Obtain a renderPassDescriptor generated from the view's drawable textures.
  MTLRenderPassDescriptor *renderPassDescriptor = view.currentRenderPassDescriptor;
  
  if(renderPassDescriptor != nil)
  {
    // Create a render command encoder.
    id<MTLRenderCommandEncoder> renderEncoder =
    [commandBuffer renderCommandEncoderWithDescriptor:renderPassDescriptor];
    renderEncoder.label = @"MyRenderEncoder";
    
    // Set the region of the drawable to draw into.
    [renderEncoder setViewport:(MTLViewport){0.0, 0.0, _viewportSize.x, _viewportSize.y, 0.0, 1.0 }];
    
    [renderEncoder setRenderPipelineState:_pipelineState];
    
    //+ NSLog(@"sizeof(triangleVertices) = %ld, sizeof(_viewportSize) = %ld", sizeof(triangleVertices), sizeof(_viewportSize));
    //+ NSLog(@"sizeof(float2) = %ld, sizeof(float4) = %ld",
    //+       sizeof(vector_float4), sizeof(vector_float4));
    
    // Pass in the parameter data.
    [renderEncoder setVertexBytes:triangleVertices
                           length:sizeof(triangleVertices)
                          atIndex:AAPLVertexInputIndexVertices];
    
    [renderEncoder setVertexBytes:&_viewportSize
                           length:sizeof(_viewportSize)
                          atIndex:AAPLVertexInputIndexViewportSize];
    
    // Draw the triangle.
    [renderEncoder drawPrimitives:MTLPrimitiveTypeTriangle
                      vertexStart:0
                      vertexCount:3];
    
    [renderEncoder endEncoding];
    
    // Schedule a present once the framebuffer is complete using the current drawable.
    [commandBuffer presentDrawable:view.currentDrawable];
  }
  
  // Finalize rendering here & push the command buffer to the GPU.
  [commandBuffer commit];
}

@end
