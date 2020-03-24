//
//  ViewController.m
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

#import "ViewController.h"
@import Metal;
#import "MetalView.h"
#import "Renderer.h"

@implementation ViewController
{
  void * _Nullable _rust_instance_ptr;
  MetalView * _view;
  Renderer * _renderer;
}

- (void)viewDidLoad {
  [super viewDidLoad];
  
  _rust_instance_ptr = nil;
  
  // Set the view to use the default device
  _view = (MetalView *)self.view;
  
  _view.device = MTLCreateSystemDefaultDevice();
  
  NSAssert(_view.device, @"Metal is not supported on this device");
  
  [_view setClearColor:MTLClearColorMake(0.0, 0.5, 1.0, 1.0)];
  
  _renderer = [[Renderer alloc] initWithMetalKitView:_view];
  
  NSAssert(_renderer, @"Renderer failed initialization");
  
  [_renderer metalView:_view drawableSizeWillChange:_view.drawableSize];
  
  _view.delegate = _renderer;
}

@end
