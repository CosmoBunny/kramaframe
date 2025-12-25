use kramaframe::{keylist::TRES16Bits, ukramaframe};

// This example demonstrates how to use kramaframe in a no_std environment
// (or simply without heap allocation) using the ukramaframe! macro.
// While this example itself uses std (for println!), the KramaFrame usage
// is strictly stack-allocated and compatible with no_std.

fn main() {
    // Define a KramaFrame with stack-allocated storage.
    // We use TRES16Bits for timing and i16 for progress resolution.
    // The macro initializes the structure with the given classes, easing functions,
    // key IDs, and durations.
    //
    // "linear_anim" uses Linear easing, has keys [1, 2], and lasts 2.0 seconds.
    // "ease_anim" uses EaseIn easing, has keys [3], and lasts 1.5 seconds.
    let mut krama = ukramaframe!(<TRES16Bits, i16, u32>
        "linear_anim" Linear [1, 2] 2.0 s;
        "ease_anim"   EaseIn [3]    1.5 s;
    );

    println!("Starting no_std example (stack allocated)...");

    // Start the animations
    krama.restart_progress("linear_anim", 1);
    krama.restart_progress("ease_anim", 3);

    let delta_time = TRES16Bits::from_millis(16); // ~60 FPS
    let total_frames = 130; // Run for about 2 seconds

    for i in 0..total_frames {
        krama.update_progress(delta_time.clone());

        // Get values for the linear animation (interpolating between 0 and 100)
        let val_linear = krama.get_value_byrange("linear_anim", 1, 0..100);

        // Get values for the ease animation (interpolating between 10 and 50)
        let val_ease = krama.get_value_byrange("ease_anim", 3, 10..50);

        // Visualizing the output
        let bar_linear = "█".repeat(val_linear as usize / 5);
        let bar_ease = "█".repeat(val_ease as usize / 2);

        println!(
            "Frame {:3}: Linear({:3}) {} | EaseIn({:3}) {}",
            i, val_linear, bar_linear, val_ease, bar_ease
        );
    }

    println!("Finished.");
}
