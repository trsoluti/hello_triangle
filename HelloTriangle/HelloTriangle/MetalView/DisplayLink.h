//
//  DisplayLink.h
//  HelloTriangle
//
//  Swift version created by Jose Canepa on 8/18/16.
//  Copyright © 2016 Jose Canepa. All rights reserved.
//  Updated by Grigory Avdyushin on 3/6/19.
//  Copyright © 2019 Grigory Avdyushin. All rights reserved.
//
//  Ported to Objective C by TR Solutions on 10/3/20.
//  Copyright © 2020 TR Solutions Pte. Ltd.
//

#import <Foundation/Foundation.h>

NS_ASSUME_NONNULL_BEGIN

@class DisplayLink;

/// @enum DisplayLinkState
/// @abstract Possible states of the display link
/// @constant DISPLAY_LINK_RUNNING Display link is generating events every sync cycle
/// @constant DISPLAY_LINK_PAUSED Display link is valid but paused
/// @constant DISPLAY_LINK_STOPPED Display link has been shut down
enum DisplayLinkState {
  DISPLAY_LINK_RUNNING,
  DISPLAY_LINK_PAUSED,
  DISPLAY_LINK_STOPPED
};

/// @abstract Error codes returned by the the display link initWithQueue:eventHandler:didFailWithError method
/// @constant FAILED_TO_CREATE_DISPLAY_LINK Attempt to connect to the timer failed
/// @constant FAILED_TO_CONNECT_DISPLAY Attempt to connect to the main display failed
/// @discussion only valid if error is not nil on return from initWithQueue:eventHandler:didFailWithError:
enum DisplayLinkError {
  FAILED_TO_CREATE_DISPLAY_LINK = -101,
  FAILED_TO_LINK_OUTPUT_HANDLER = -102,
  FAILED_TO_CONNECT_DISPLAY = -103
};

/// @typedef TimerEventBlock
/// @abstract The signature of the callback executed every time the display sync timer goes off.
/// @discussion The event block is run in the main thread.
typedef void (^ TimerEventBlock)(DisplayLink *);

/// @interface DisplayLink
/// @abstract An object which manages connection to the display sync timer
/// @discussion
/// Whenever the display indicates it is about to sync,
/// this object will call the provided event handler on the main thread.
@interface DisplayLink : NSObject

/// @method initWithQueue:eventHandler:didFailWithError:
/// @abstract Initialization method.
/// @param queue the dispatch queue to use to wake up the handler (usually the main queue)
/// @param timerEventBlock the handler to call when the display is about to sync
/// @param error a point to the address of an error variable. This address will be filled in
/// if the method is not able to initialize the DisplayLink object. Otherwise it will be nil.
- (instancetype)initWithQueue: (dispatch_queue_t _Nonnull)queue
                 eventHandler: (TimerEventBlock) timerEventBlock
             didFailWithError: (NSError * _Nullable * _Nonnull) error;
/// Starts the timer
- (void) start;
//
/// Pauses the timer, can be restarted afterwards
- (void) pause;
//
/// Cancels the timer
- (void) stop;

@end

NS_ASSUME_NONNULL_END
