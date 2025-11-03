![logo](./logo.svg)

A simple, generic, and flexible keyframe animation library for Rust.

[![Crates.io](https://img.shields.io/crates/v/kramaframe.svg)](https://crates.io/crates/kramaframe)
[![Docs.rs](https://docs.rs/kramaframe/badge.svg)](https://docs.rs/kramaframe)

`KramaFrame` provides a straightforward way to manage and update multiple animations, map their progress to value ranges, and apply various easing functions for smooth and dynamic transitions.

## Features

*   Manage multiple animations using string-based class names and numeric IDs.
*   A rich set of built-in easing functions:
    *   `Linear`
    *   `EaseIn`, `EaseOut`, `EaseInOut`
    *   `Steps(n)` for stepped animations
    *   Custom `CubicBezier` curves
*   Generic over time (`TRES`) and progress (`PRES`) data types for maximum flexibility.
*   Easily map animation progress (from 0.0 to 1.0) to any numerical output range.
*   Play animations forwards or in reverse.
*   Update all active animations with a single call in your game or application loop.

## Getting Started

Add `kramaframe` to your project's `Cargo.toml`:
```toml
[dependencies]
kramaframe = "0.1.4"
```

Or, add it via the command line:
```sh
cargo add kramaframe
```

## Usage

Here's a basic example of animating a value from 80 to 100 over a set duration.

```rust
use kramaframe::{BTclasslist, BTframelist, KramaFrame, keyframe::KeyFrameFunction};

fn main() {
    // 1. Create a new KramaFrame instance.
    // The type parameters define the storage for animation classes and instances.
    // Here, BTframelist<_, i32> uses a default time representation (f64) and i32 for progress.
    let mut kramaframe: KramaFrame<BTclasslist, BTframelist<_, i32>> = KramaFrame::default();

    // 2. Define an animation "class" with an easing function.
    // We'll call it "animation1" and use Linear easing for a constant speed.
    kramaframe.extend_iter_classlist([("animation1", KeyFrameFunction::Linear)]);

    // 3. Create a specific instance of the animation.
    // We give it a unique ID (1) and a duration (1.0 in abstract units).
    kramaframe.insert_new_id("animation1", 1, 1.0);

    // 4. Start the animation. This sets its progress to the beginning.
    kramaframe.restart_progress("animation1", 1);

    // 5. In your application's update loop, advance the animation time.
    println!("--- Forward Animation ---");
    for _ in 0..=120 {
        // Update all animations by a delta time.
        // Here, we simulate 120 frames for a total duration of 1.0.
        kramaframe.update_progress(1.0 / 120.0);

        // 6. Get the current animated value.
        // We map the animation's progress (0.0 to 1.0) to a target range (80.0 to 100.0).
        let progress = kramaframe.get_progress_f32("animation1", 1).unwrap_or(0.0);
        let value = kramaframe.get_value_byrange_inclusive("animation1", 1, 80f32..=100f32);

        println!("progress= {:.2}, value = {:.2}", progress, value);
    }

    // 7. You can also reverse the animation.
    kramaframe.reverse_animate("animation1", 1);

    println!("\n--- Reverse Animation ---");
    for _ in 0..=120 {
        kramaframe.update_progress(1.0 / 120.0);
        let progress = kramaframe.get_progress_f32("animation1", 1).unwrap_or(0.0);
        let value = kramaframe.get_value_byrange_inclusive("animation1", 1, 80f32..=100f32);

        println!("progress= {:.2}, value = {:.2}", progress, value);
    }
}
```

## Easing Functions (`KeyFrameFunction`)

`KramaFrame` uses `KeyFrameFunction` to control the rate of change of an animation, allowing for more natural and interesting motion.

### Available Functions

*   `KeyFrameFunction::Linear`: Constant speed.
*   `KeyFrameFunction::EaseIn`: Starts slow, then accelerates.
*   `KeyFrameFunction::EaseOut`: Starts fast, then decelerates.
*   `KeyFrameFunction::EaseInOut`: Starts and ends slow, with acceleration in the middle.
*   `KeyFrameFunction::Steps(n)`: Divides the animation into `n` discrete, instantaneous steps.
*   `KeyFrameFunction::new_cubic_bezier_f32(x1, y1, x2, y2)`: Defines a custom animation curve using Cubic Bezier control points, similar to CSS `cubic-bezier()`.

### Example

You can define multiple animation classes, each with its own easing function.

```rust
use kramaframe::{KramaFrame, keyframe::KeyFrameFunction, keylist::KeyList};

let mut kramaframe = KramaFrame::default();

// Define different animation classes
kramaframe.extend_iter_classlist([
    ("linear", KeyFrameFunction::Linear),
    ("easein", KeyFrameFunction::EaseIn),
    ("easeinout", KeyFrameFunction::EaseInOut),
    ("bounce", KeyFrameFunction::new_cubic_bezier_f32(0.0, 1.26, 1.0, -0.79)),
    ("staircase", KeyFrameFunction::Steps(5)),
]);

// Create instances for each animation class with a duration of 1.0
kramaframe.insert_new_id("linear", 1, 1.0);
kramaframe.insert_new_id("easein", 1, 1.0);
// ... and so on for the other classes

// In your update loop, you can get values for each:
kramaframe.update_progress(1.0 / 60.0); // ~60 FPS

let linear_val = kramaframe.get_value_byrange("linear", 1, 0..100);
let easein_val = kramaframe.get_value_byrange("easein", 1, 0..100);

println!("Linear: {}, EaseIn: {}", linear_val, easein_val);
```

## More Examples

You can find more detailed examples in the `/examples` directory of the repository:

*   `iterrange.rs`: A complete, runnable version of the basic usage example.
*   `allkeyframe.rs`: Demonstrates and visualizes all available `KeyFrameFunction` types.
*   `reverse.rs`: Demonstrates how to reverse the direction of an animation instance.
*   `reverserange.rs`: Demonstrates how to reverse the range of an animation instance.
*   `generic.rs`: Demonstrates how to use generic types with `KramaFrame`.

## License

This project is licensed under

*   MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

TODO:
- [ ] Add more examples
- [ ] `no_std` support
- ... can be more from commit.
