//! Timer to sync metal updates with display sync
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

use cocoa::base::id;
use objc::runtime::Object;
use std::fmt::Formatter;
use std::error::Error;
use std::ffi::c_void;
use std::os::raw::{c_int, c_ulonglong, c_ulong};
use cocoa::quartzcore::CVTimeStamp;
use std::ptr::null;

pub type CVDisplayLinkRef = *const c_void;
pub type dispatch_source_t = id;
pub type uintptr_t = *const c_void;
pub type dispatch_queue_t = id;
pub type CGDirectDisplayID = *const c_void;
pub type CVReturn = c_int;
pub type CVOptionFlags = c_ulonglong;
pub type CVDisplayLinkOutputCallback = *const extern "C" fn(display_link: CVDisplayLinkRef, now: *const CVTimeStamp, output_time: *const CVTimeStamp, flags_in: CVOptionFlags, flags_out: *mut CVOptionFlags, display_link_context: *const c_void) -> CVReturn;
pub type dispatch_object_t = id;

// CVReturn.h:    kCVReturnSuccess = 0
static kCVReturnSuccess: c_int = 0;

#[link(name = "QuartzCore", kind = "framework")]
extern {
    static _dispatch_source_type_data_add: c_void;
    // dispatch_source_t dispatch_source_create(dispatch_source_type_t type, uintptr_t handle, unsigned long mask, dispatch_queue_t queue);
    fn dispatch_source_create(dispatch_type: &c_void, handle: uintptr_t, mask: c_ulong, queue: dispatch_queue_t) -> dispatch_source_t;
    // CGDirectDisplayID CGMainDisplayID(void);
    fn CGMainDisplayID() -> CGDirectDisplayID;
    // CVReturn CVDisplayLinkCreateWithCGDisplay(CGDirectDisplayID displayID, CVDisplayLinkRef  _Nullable *displayLinkOut);
    fn CVDisplayLinkCreateWithCGDisplay(display_id: CGDirectDisplayID, display_link_out: &mut CVDisplayLinkRef) -> CVReturn;
    // CVReturn CVDisplayLinkSetOutputCallback(CVDisplayLinkRef displayLink, CVDisplayLinkOutputCallback callback, void *userInfo);
    fn CVDisplayLinkSetOutputCallback(display_link: CVDisplayLinkRef, callback: CVDisplayLinkOutputCallback, user_info: *const c_void) -> CVReturn;
    // void dispatch_resume(dispatch_object_t object);
    fn dispatch_resume(object: dispatch_object_t);
    // CVReturn CVDisplayLinkStart(CVDisplayLinkRef displayLink);
    fn CVDisplayLinkStart(display_link: CVDisplayLinkRef) -> CVReturn;
    // CVReturn CVDisplayLinkStop(CVDisplayLinkRef displayLink);
    fn CVDisplayLinkStop(display_link: CVDisplayLinkRef) -> CVReturn;
    // oid dispatch_suspend(dispatch_object_t object);
    fn dispatch_suspend(object: dispatch_object_t);
    fn dispatch_source_merge_data(source: dispatch_source_t, data: c_ulong);
}
#[link(name="System", kind="framework")]
extern {
    // oid dispatch_cancel(dispatch_object_t object);
    fn dispatch_source_cancel(object: dispatch_object_t);
}
// typedef void (*dispatch_function_with_user_data_t)(dispatch_source_t _Nonnull , void *_Nullable);
pub trait dispatch_function_with_user_data_trait: Fn(dispatch_source_t, *const c_void){}
pub type dispatch_function_with_user_data_t = extern "C" fn(source: dispatch_source_t, user_data: *const c_void);
#[link(name = "GlueLib", kind = "dylib")]
extern {
    // void dispatch_source_set_event_handler_f_with_user_data
    // (dispatch_source_t _Nonnull,
    //  dispatch_function_with_user_data_t _Nullable,
    //   void * _Nullable
    //   );
    fn dispatch_source_set_event_handler_f_with_user_data(
        source: dispatch_source_t,
        handler: dispatch_function_with_user_data_t,
        user_data: *const c_void,
    );
}


// from usr/include/dispatch/source.h:
// #define DISPATCH_SOURCE_TYPE_DATA_ADD (&_dispatch_source_type_data_add)
static DISPATCH_SOURCE_TYPE_DATA_ADD: &c_void = unsafe { &_dispatch_source_type_data_add };

pub enum DisplayLinkState {
    Running,
    Paused,
    Stopped,
}

#[derive(Debug)]
pub enum DisplayLinkError {
    FailedToCreateTimer,
    FailedToConnectToDisplay,
}
impl std::fmt::Display for DisplayLinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FailedToCreateTimer => write!(f, "Failed to create timer"),
            Self::FailedToConnectToDisplay => write!(f, "Failed to connect to display"),
        }
    }
}
impl Error for DisplayLinkError{}

pub struct DisplayLink {
    pub state: DisplayLinkState,
    pub is_running: bool,
    _display_link: CVDisplayLinkRef,
    _source: dispatch_source_t,
    _timer_event_callback: Option<Box<fn(&mut Object)>>,
    _caller: id,
}

impl DisplayLink {
    pub fn new_with_queue_and_callback(queue: dispatch_queue_t, timer_event_callback: fn(&mut Object), caller: id) -> Result<Box<Self>, DisplayLinkError> {
        println!("In DisplayLink::new(), caller = {:?}", caller);
        let source = unsafe { dispatch_source_create(
            DISPATCH_SOURCE_TYPE_DATA_ADD,
            null(),
            0,
            queue
        ) };
        let mut display_link_ref: CVDisplayLinkRef = null();
        unsafe {
            let return_code = CVDisplayLinkCreateWithCGDisplay(
                CGMainDisplayID(),
                &mut display_link_ref
            );
            if return_code != kCVReturnSuccess {
                return Err(DisplayLinkError::FailedToConnectToDisplay);
            }
        }
        let return_code = unsafe { CVDisplayLinkSetOutputCallback(
            display_link_ref,
            display_link_callback as CVDisplayLinkOutputCallback,
            source as *const c_void,
        ) };
        if return_code != kCVReturnSuccess {
            return Err(DisplayLinkError::FailedToCreateTimer);
        }
        let display_link = Box::new(DisplayLink {
            state: DisplayLinkState::Stopped,
            is_running: false,
            _display_link: display_link_ref,
            _source: source,
            /*_timer_event_block: event_handler,*/
            /*_timer_event_callback: &|| {}*/
            _timer_event_callback: Some(Box::new(timer_event_callback.clone())),
            _caller: caller,
        });
        unsafe { dispatch_source_set_event_handler_f_with_user_data(
            source,
            dispatch_event_handler_f::<fn(id)>,
            display_link.as_ref() as *const _ as *const c_void,
        ) };
        Ok(display_link)
    }
    pub fn start(&mut self) {
        if self.can_enter_state(DisplayLinkState::Running) {
            unsafe {
                let _ = CVDisplayLinkStart(self._display_link);
                dispatch_resume(self._source);
            }
            self.state = DisplayLinkState::Running;
        }
    }
    pub fn stop(&mut self) {
        if let DisplayLinkState::Running = self.state {
            unsafe {
                let _ = CVDisplayLinkStop(self._display_link);
                dispatch_source_cancel(self._source);
            }
        }
        assert!(self.can_enter_state(DisplayLinkState::Stopped));
        self.state = DisplayLinkState::Stopped
    }
    #[allow(unused)]
    pub fn pause(&mut self) {
        if self.can_enter_state(DisplayLinkState::Paused) {
            unsafe {
                let _ = CVDisplayLinkStop(self._display_link);
                dispatch_suspend(self._source)
            }
            self.state = DisplayLinkState::Paused;
        }
    }

    fn can_enter_state(&self, next_state: DisplayLinkState) -> bool {
        match self.state {
            DisplayLinkState::Running =>
                match next_state {
                    DisplayLinkState::Paused => true,
                    DisplayLinkState::Stopped => true,
                    _ => false
                },
            DisplayLinkState::Paused =>
                match next_state {
                    DisplayLinkState::Running => true,
                    DisplayLinkState::Stopped => true,
                    _ => false
                },
            DisplayLinkState::Stopped =>
                match next_state {
                    DisplayLinkState::Running => true,
                    _ => false
                }
        }
    }
}

extern "C" fn display_link_callback(
    _display_link: CVDisplayLinkRef,
    _now: &CVTimeStamp,
    _output_time: &CVTimeStamp,
    _flags_in: CVOptionFlags,
    _flags_out: &mut CVOptionFlags,
    _display_link_context: *const c_void,
) -> CVReturn {
    let source = _display_link_context as dispatch_source_t;
    unsafe { dispatch_source_merge_data(source, 1) };
    kCVReturnSuccess
}

#[allow(unused)]
extern "C" fn dispatch_event_handler_f<F>(_source: dispatch_source_t, user_data: *const c_void)
    where F: Fn(id)
{
    //+ println!("In dispatch_event_handler_f, user data = {:?}", user_data);
    unsafe {
        let user_display_link_ptr:*const DisplayLink = user_data.cast();
        let opt_user_display_link = user_display_link_ptr.as_ref();
        match opt_user_display_link {
            Some(display_link_p) => match &display_link_p._timer_event_callback {
                Some(f) => {
                    let p = &display_link_p._caller;
                    let q = p.as_mut().unwrap();
                    //+ println!("  p={:?}, q={:?}", p, q);
                    f(q)
                },
                None => println!("No handler function!")
            }
            None => println!("No user data!")
        }
    }
}
