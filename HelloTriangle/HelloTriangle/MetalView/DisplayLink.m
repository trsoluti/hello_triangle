//
//  DisplayLink.m
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

#import "DisplayLink.h"
@import CoreVideo;
#import "GlueLib.h"

@implementation DisplayLink
{
  /// The link to the Core Video display
  CVDisplayLinkRef _displayLink;
  /// The source of our dispatch event
  dispatch_source_t _source;
  /// The handler to call on a timer event
  TimerEventBlock _timerEventBlock;
  /// Whether we are stopped, running or paused
  enum DisplayLinkState _currentState;
}

#pragma mark - initializer

- (instancetype)initWithQueue:(dispatch_queue_t)queue
                 eventHandler:(TimerEventBlock)timerEventBlock
             didFailWithError:(NSError * _Nullable __autoreleasing *)error
{
  self = [super init];
  if (self) {
    _timerEventBlock = timerEventBlock;
    _source = dispatch_source_create(DISPATCH_SOURCE_TYPE_DATA_ADD, 0, 0, queue);
    
    // Get a display link pointer
    CVDisplayLinkRef displayLinkRef;
    CGDirectDisplayID mainDisplay = CGMainDisplayID();
    CVDisplayLinkCreateWithCGDisplay(mainDisplay, &displayLinkRef);
    
    if (displayLinkRef == nil) {
      if (error != nil) {
        NSString *domain = @"com.trsolutions.DisplayLink.InitializationErrors";
           NSString *desc = NSLocalizedString(@"Unable to create display link", @"");
           NSDictionary *userInfo = @{ NSLocalizedDescriptionKey : desc };
        
          *error = [NSError errorWithDomain:domain
                                       code: FAILED_TO_CREATE_DISPLAY_LINK
                                   userInfo:userInfo];
      }
    }
    
    _displayLink = displayLinkRef;
    CVReturn returnCode = CVDisplayLinkSetOutputCallback(_displayLink, displayLinkCallback, (__bridge void * _Nullable)(_source));
    
    if (returnCode != kCVReturnSuccess) {
        NSString *domain = @"com.trsolutions.DisplayLink.InitializationErrors";
           NSString *desc = NSLocalizedString(@"Unable to link output handler", @"");
           NSDictionary *userInfo = @{ NSLocalizedDescriptionKey : desc };
        
          *error = [NSError errorWithDomain:domain
                                       code:FAILED_TO_LINK_OUTPUT_HANDLER
                                   userInfo:userInfo];
    }

    // Note: the following block is the best way to handle the event,
    // but I'm not able (yet) to figure out how to convert obcj blocks
    // to persistent Rust (objc) closures,
    // so we use a callback function with user data that's easier to
    // port.
    //
    // Best way, with block:
    //x dispatch_source_set_event_handler(_source, ^{
    //x   //+NSLog(@"in dispatch source event handler");
    //x   if (self != nil) {
    //x     self->_timerEventBlock(self);
    //x   }
    //x });
    //
    // Most portable way:
    dispatch_source_set_event_handler_f_with_user_data(_source, &dispatch_event_handler_f, (__bridge void * _Nullable)(self));
    
    _currentState = DISPLAY_LINK_STOPPED;
    *error = nil;
  }
  return self;
}

- (void) dealloc {
  // Release of suspended timer causes a crash in realtime!
  if (_currentState != DISPLAY_LINK_RUNNING) {
    dispatch_resume(_source);
  }
}

#pragma mark - change the timer state

- (void) start {
  if ([self canEnterState:DISPLAY_LINK_RUNNING]) {
    CVDisplayLinkStart(_displayLink);
    dispatch_resume(_source);
  } else {
    assert(false);
  }
}

- (void) pause {
  if ([self canEnterState:DISPLAY_LINK_PAUSED]) {
    CVDisplayLinkStop(_displayLink);
    dispatch_suspend(_source);
  } else {
    assert(false);
  }
}

- (void) stop {
  if (_currentState == DISPLAY_LINK_RUNNING) {
    CVDisplayLinkStop(_displayLink);
    dispatch_cancel(_source);
  } else if (![self canEnterState:DISPLAY_LINK_STOPPED]) {
    assert(false);
  }
}

#pragma mark - support methods

- (BOOL) canEnterState:(enum DisplayLinkState)nextState {
  switch (_currentState) {
  case DISPLAY_LINK_RUNNING:
      return nextState == DISPLAY_LINK_PAUSED || nextState == DISPLAY_LINK_STOPPED;
  case DISPLAY_LINK_PAUSED:
      return nextState == DISPLAY_LINK_RUNNING || nextState == DISPLAY_LINK_STOPPED;
  case DISPLAY_LINK_STOPPED:
      return nextState == DISPLAY_LINK_RUNNING;
  }
}

static CVReturn displayLinkCallback
 (
    CVDisplayLinkRef displayLink,
    const CVTimeStamp* now,
    const CVTimeStamp* outputTime,
    CVOptionFlags flagsIn,
    CVOptionFlags* flagsOut,
    void* displayLinkContext
    )
{
  //+NSLog(@"in displayLinkCallback");
  NSObject<OS_dispatch_source> * source = (__bridge NSObject<OS_dispatch_source> *)displayLinkContext;
  dispatch_source_merge_data(source, 1);
  return kCVReturnSuccess;
}

static void dispatch_event_handler_f(dispatch_source_t _Nonnull source, void* _Nullable user_data) {
  DisplayLink *displayLink = (__bridge DisplayLink *) user_data;
  displayLink->_timerEventBlock(displayLink);
}

@end
