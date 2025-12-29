use kramaframe::{btkramaframe, keylist::TRES32Bits};

// This example demonstrates how to use kramaframe in a no_std environment
// WITH the 'alloc' feature enabled.
// This allows using BTreeMap-based KramaFrame (heap allocated) even without std.
//
// To run this check (simulated no_std):
// cargo run --example alloc_nostd --no-default-features --features no_std,alloc
//
// Note: This example uses println! which is from std, but the KramaFrame usage
// relies on alloc::collections::BTreeMap when configured with no_std + alloc.

fn main() {
    // Define a KramaFrame using the macro.
    // In no_std + alloc mode, this uses alloc::collections::BTreeMap.
    // We use i32 for progress resolution (Eq required).
    let mut krama = btkramaframe!(<TRES32Bits, i32>
        "fade_in" EaseIn [1, 2] 2.0 s;
        "move"    Linear [10]   5.0 s;
    );

    println!("Starting no_std + alloc example...");

    krama.restart_progress("fade_in", 1);
    krama.restart_progress("move", 10);

    // Simulate a few frames
    for i in 0..5 {
        let dt = TRES32Bits::from_seconds(0.5);
        krama.update_progress(dt);

        let fade = krama.get_value_byrange("fade_in", 1, 0.0..1.0);
        let pos = krama.get_value_byrange("move", 10, 0.0..100.0);

        println!("Frame {}: Fade {:.2}, Pos {:.2}", i, fade, pos);
    }

    println!("Finished.");
}
