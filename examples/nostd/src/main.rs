use kramaframe::{keylist::TRES16Bits, ukramaframe};

fn main() {
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
