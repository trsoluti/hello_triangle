//
//  MetalView.m
//  HelloTriangle
//
//  Created by TR Solutions on 22/3/20.
//  Copyright © 2020 TR Solutions Pte. Ltd.
//  Licensed under Apache 2.0 and MIT
//  See appropriate LICENCE files for details.
//

#import "MetalView.h"
#import "DisplayLink.h"
#import "GlueLib.h"

@implementation MetalView
{
  /// @var _device
  /// @abstract The Metal device on which the view operates
  id<MTLDevice> _device;
  /// @var _clearColor
  /// @abstract The color with which to paint the background before calling drawInMetalView:
  MTLClearColor _clearColor;
  /// @var _timer
  /// @abstract The DisplayLink timer which will call us every time the display is about to sync
  DisplayLink * _timer;
  /// @var _delegate
  /// @abstract The delegate that will be managing the graphics content
  id<MetalViewDelegate> _delegate;
  /// @var _drawableSize
  /// @abstract The current size of the drawable area
  CGSize _drawableSize;
}

#pragma mark - Getters and Setters

- (id<MTLDevice>) device {
  return _device;
}
- (void) setDevice:(id<MTLDevice>)newDevice {
  CAMetalLayer * _Nullable metalLayer = (CAMetalLayer *) [self layer];
  if (metalLayer != nil) {
    [metalLayer setDevice:newDevice];
  }
  _device = newDevice;
}

- (id<MetalViewDelegate>) delegate {
  return _delegate;
}
- (void) setDelegate:(id<MetalViewDelegate>) newDelegate {
  if (newDelegate != nil) {
    [_timer start];
  } else {
    [_timer stop];
  }
  _delegate = newDelegate;
}

/// @method mtlLayer
/// @abstract The underlying layer cast as a CAMetalLayer
/// @return the (possibly nil) underlying drawing area as a CAMetalLayer
- (CAMetalLayer * _Nullable)mtlLayer {
  return (CAMetalLayer *)[self layer];
}

-(BOOL) wantsUpdateLayer { return YES; }

/*! Returns a Metal-compatible layer. */
+(Class) layerClass { return [CAMetalLayer class]; }

- (MTLClearColor)clearColor {
  return self->_clearColor;
}
- (void)setClearColor:(MTLClearColor)newColor {
  self ->_clearColor = newColor;
}

#pragma mark - initializer

- (instancetype)initWithCoder:(NSCoder *)coder {
  self = [super initWithCoder:coder];
  if (self) {
    [self setEnableSetNeedsDisplay:false];
    _clearColor = MTLClearColorMake(0.0, 0.0, 0.0, 1.0);
    NSError * displayLinkCreateError = nil;
    _timer = [[DisplayLink alloc] initWithQueue:dispatch_get_main_queue_not_inline()
                                   eventHandler:^(DisplayLink * _Nonnull _displayLink) {
      [self setDrawableSize:[self convertSizeToBacking:[self frame].size]];
      [self setUpDelegateDrawingState];
      if (self.delegate != nil) {
        [self.delegate drawInMetalView:self];
      }
    } didFailWithError:&displayLinkCreateError];
    
    if (displayLinkCreateError != nil) {
      NSLog(@"Display link error: %@", displayLinkCreateError);
    } else {
      NSLog(@"No error");
    }
    
    [self setDrawableSize:[self convertSizeToBacking:[self frame].size]];
    self.wantsLayer = true;
  }
  return self;
}

/** If the wantsLayer property is set to YES, this method will be invoked to return a layer instance. */
-(CALayer*) makeBackingLayer {
   CALayer* layer = [self.class.layerClass layer];
   CGSize viewScale = [self convertSizeToBacking: CGSizeMake(1.0, 1.0)];
   layer.contentsScale = MIN(viewScale.width, viewScale.height);
   return layer;
}

#pragma mark - items used in delgate's draw method:

- (MTLPixelFormat) colorPixelFormat {
  if ([self mtlLayer] != nil) {
    return [[self mtlLayer] pixelFormat];
  } else {
    return MTLPixelFormatBGRA8Unorm;
  }
}

- (CGSize) drawableSize {
  return _drawableSize;
}
- (void) setDrawableSize:(CGSize)newDrawableSize {
  if (newDrawableSize.height != _drawableSize.height
      || newDrawableSize.width != _drawableSize.width) {
    if ([self mtlLayer] != nil) {
      [[self mtlLayer] setDrawableSize:newDrawableSize];
    }
    if (_delegate != nil) {
      [_delegate metalView:self drawableSizeWillChange:newDrawableSize];
    }
    _drawableSize = newDrawableSize;
  }
}

/// Used by the timer to prepare the drawing state vars for a new delegate
- (void) setUpDelegateDrawingState {
  _currentRenderPassDescriptor = [[MTLRenderPassDescriptor alloc] init];
  if ([self mtlLayer]) {
    _currentDrawable = [[self mtlLayer] nextDrawable];
  }
  if (_currentRenderPassDescriptor != nil) {
    MTLRenderPassColorAttachmentDescriptor * _Nullable colorAttachments = [_currentRenderPassDescriptor colorAttachments][0];
    if (colorAttachments != nil) {
      if (_currentDrawable != nil) {
        colorAttachments.texture = _currentDrawable.texture;
      }
      colorAttachments.loadAction = MTLLoadActionClear;
      colorAttachments.clearColor = _clearColor;
    }
  }
}

@end
