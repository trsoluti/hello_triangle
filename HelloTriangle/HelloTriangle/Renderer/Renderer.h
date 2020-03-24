//
//  Renderer.h
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

#import <Foundation/Foundation.h>
#import "MetalView.h"

NS_ASSUME_NONNULL_BEGIN

/// @interface Renderer
/// @abstract A class which is responsible for rendering the graphics on the Metal View
@interface Renderer : NSObject<MetalViewDelegate>
/// @method initWithMetalKitView:
/// @abstract creates a new Renderer with the given Metal view.
/// @param metalView the view in which the renderer will draw.
- (nonnull instancetype)initWithMetalKitView:(nonnull MetalView *)metalView;

@end

NS_ASSUME_NONNULL_END
