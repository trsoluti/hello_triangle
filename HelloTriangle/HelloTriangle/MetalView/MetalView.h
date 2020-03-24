//
//  MetalView.h
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

#import <Cocoa/Cocoa.h>
#import <QuartzCore/CAMetalLayer.h>
#import <Metal/Metal.h>

NS_ASSUME_NONNULL_BEGIN

@class MetalView;

/// @protocol MetalViewDelegate
///
/// @abstract
/// A set of methods that delegates of Metal View objects must implement.
@protocol MetalViewDelegate <NSObject>

/// @method metalView:(MetalView *)view drawableSizeWillChange:(CGSize)size
///
/// @abstract
/// Called when the size of the window is about to change
/// @param view the Metal view which is managing the drawing
/// @param size the new size of the window
- (void)metalView:(MetalView *)view drawableSizeWillChange:(CGSize)size;

/// @method drawInMetalView:
/// @abstract Called when the Metal View expects some drawing
/// @param view the MetalView which is managing the drawing
/// @discussion
/// Before calling this method, MetalVIew will set up
/// the following instance variables for your use:
///
/// - currentRenderPassDescriptor
///
/// - currentDrawable
- (void)drawInMetalView:(MetalView *)view;

@end

/// @interface MetalView
/// @abstract A class to handle the interface to Metal while operating as an NSView.
/// @discussion
/// This class provides an interface similar to MetalKit's MTKView. It syncs with the display
/// and arranges to call the delegate back whenever the main display is about to sync.
@interface MetalView : NSView
/// @property device
/// @abstract The Metal device on which the view operates
@property id<MTLDevice> device;
/// @property delegate
/// @abstract The delegate that will be managing the graphics content
/// @discussion if the delegate is nil, the display sync is turned off
@property (retain, nullable) id<MetalViewDelegate> delegate;
/// @property enableSetNeedsDisplay
/// @abstract Whether or not to enable Set Needs Display
/// @discussion this variable has no effect, but is included for compatibility with MTKView
@property (atomic) BOOL enableSetNeedsDisplay;
/// @property colorPixelFormat
/// @abstract The pixel format of the underlying display layer.
@property (atomic, readonly) MTLPixelFormat colorPixelFormat;
/// @property currentRenderPassDescriptor
/// @abstract The descriptor of the current render pass
/// @discussion valid only during drawInMetalView:
@property (retain, nullable) MTLRenderPassDescriptor * currentRenderPassDescriptor;
/// @property currentDrawable
/// @abstract The current Metal drawable
/// @discussion only valid during drawInMetalView:
@property (retain, nullable) id<CAMetalDrawable> currentDrawable;
/// @property drawableSize
/// @abstract The current size of the drawing area
/// @discussion if this size changes, the view will call metalView:drawableSizeWillChange:
@property (atomic) CGSize drawableSize;
/// @property clearColor
/// @abstract The color with which to paint the background before calling drawInMetalView:
@property (atomic) MTLClearColor clearColor;
@end

NS_ASSUME_NONNULL_END
