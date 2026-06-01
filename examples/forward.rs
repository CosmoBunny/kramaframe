use kramaframe::{
    BTframelist, KramaFrame,
    keyframe::KeyFrameFunction,
    keylist::{KeyList, TRES16Bits},
};

fn main() {
    let mut kramaframe: KramaFrame<_, BTframelist<TRES16Bits, i32>> = KramaFrame::default();

    kramaframe.extend_iter_classlist([("linear", KeyFrameFunction::Linear)]);

    kramaframe
        .framelist
        .extend([("linear", KeyList::new(1, TRES16Bits::from_millis(1000)))]);

    println!("--- Starting Forward ---");
    kramaframe.restart_progress("linear", 1);
    for i in 0..30 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let val = kramaframe.get_value_byrange("linear", 1, 0..100u32);
        if i % 10 == 0 {
            println!("Frame {}: {}", i, val);
        }
    }

    println!("--- Reversing Mid-way ---");
    kramaframe.reverse_animate("linear", 1);
    for i in 0..15 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let val = kramaframe.get_value_byrange("linear", 1, 0..100u32);
        if i % 5 == 0 {
            println!("Frame {}: {}", i, val);
        }
    }

    println!("--- Explicitly Setting Forward ---");
    kramaframe.forward_animate("linear", 1);
    for i in 0..30 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let val = kramaframe.get_value_byrange("linear", 1, 0..100u32);
        if i % 10 == 0 {
            println!("Frame {}: {}", i, val);
        }
    }
}
