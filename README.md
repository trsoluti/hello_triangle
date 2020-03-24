# hello_triangle
The MacOS HelloTriangle tutorial written first in Objective C then ported to Rust

I can't see much of a use-case for this code, but here it is.

## Description

I created a Glue Lib dynamic library to encapsulate certain Objective-C macros and functions that aren't accessible from Rust.

I took Apple's Hello Triangle example code from [here](https://developer.apple.com/documentation/metal/using_a_render_pipeline_to_render_primitives?language=objc) and removed the portions for other targets.

I modified the code to:

- use a custom hand-coded version to MTLView instead of the one from the MetalKit framework (reduce # of frameworks used)
- use the glue lib methods instead of calling the Cocoa framework directly (to show porting)

Then I created a Rust version of the project, which:

- registers the needed classes with Objective C classes, iVars and methods with the Obcj Runtime
- hands control to `NSApplicationMain` to set up the threads, read the storyboard, and do all the Cocoa stuff
- on setting up the view:
  - registers the Objective C instance with a global Rust dictionary so we can go back and forth
  - does the same things that the Objective C view does but with as much of it as possible in Rust
- when the view is loaded:
  - creates a Rust renderer and sets up the renderer to repaint the screen each time it refreshes.

In order to compile and run the Rust version, I need to:
- copy the `libGlueLib.dylib` file to somewhere in the rust compiler's library search path.
- copy the `main.storyboardc` **folder** to the run folder of the command (target/debug in IntelliJ)
- copy the `default.metallib` file to the run folder of the command

Then it runs from IntelliJ (although it doesn't do a main menu or appear in the MacOS task list)

## Licensing:

The code is dual-licensed under the **Apache-2.0** and **MIT** licenses. Please see the appropriate license files for details.

Portions of the code are covered under a non-restrictive Apple Licence (LICENCE-APPLE).

The original (Swift) source code for DisplayLink was taken from [here](https://gist.github.com/avdyushin/35a5c6d92a08c7e31dfb1961f7d9db3e) and ported to Objective C then to Rust. I'm still trying to clarify the license of this portion.


