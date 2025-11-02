use kramaframe::{
    BTframelist, KramaFrame,
    keyframe::KeyFrameFunction,
    keylist::{KeyList, TRES16Bits},
};

fn main() {
    let mut kramaframe: KramaFrame<_, BTframelist<TRES16Bits, i32>> = KramaFrame::default();

    kramaframe.extend_iter_classlist([
        ("linear", KeyFrameFunction::Linear),
        ("ease", KeyFrameFunction::Ease),
        (
            "cubic",
            KeyFrameFunction::new_cubic_bezier_f32(0., 0.5, 1., -0.35),
        ),
    ]);

    kramaframe.framelist.extend([
        ("linear", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("ease", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("cubic", KeyList::new(1, TRES16Bits::from_millis(1000))),
    ]);

    // Linear
    kramaframe.restart_progress("linear", 1);
    for _ in 0..=61 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let linear = kramaframe.get_value_byrange("linear", 1, 0..90u32);
        println!("linear : {} : {}", linear, "█".repeat(linear as usize));
    }
    kramaframe.reverse_animate("linear", 1);
    for _ in 0..=61 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let linear = kramaframe.get_value_byrange("linear", 1, 0..90u32);
        println!("linear : {} : {}", linear, "█".repeat(linear as usize));
    }

    // Ease
    kramaframe.restart_progress("ease", 1);
    for _ in 0..=61 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let ease = kramaframe.get_value_byrange("ease", 1, 0..90u32);
        println!("ease : {} : {}", ease, "█".repeat(ease as usize));
    }
    kramaframe.reverse_animate("ease", 1);
    for _ in 0..=61 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let ease = kramaframe.get_value_byrange("ease", 1, 0..90u32);
        println!("ease : {} : {}", ease, "█".repeat(ease as usize));
    }

    // Cubic
    kramaframe.restart_progress("cubic", 1);
    for _ in 0..=62 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let cubic = kramaframe.get_value_byrange("cubic", 1, 0..90u32);
        println!("cubic : {} : {}", cubic, "█".repeat(cubic as usize));
    }
    kramaframe.reverse_animate("cubic", 1);
    for _ in 0..=62 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let cubic = kramaframe.get_value_byrange("cubic", 1, 0..90u32);
        println!("cubic : {} : {}", cubic, "█".repeat(cubic as usize));
    }

    // Or call reverse_start to jump to the end.
    kramaframe.reverse_start("linear", 1);
    for _ in 0..=62 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let linear = kramaframe.get_value_byrange("linear", 1, 0..90u32);
        println!("linear : {} : {}", linear, "█".repeat(linear as usize));
    }
    kramaframe.reverse_start("ease", 1);
    for _ in 0..=62 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let ease = kramaframe.get_value_byrange("ease", 1, 0..90u32);
        println!("ease : {} : {}", ease, "█".repeat(ease as usize));
    }
    kramaframe.reverse_start("cubic", 1);
    for _ in 0..=62 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let cubic = kramaframe.get_value_byrange("cubic", 1, 0..90u32);
        println!("cubic : {} : {}", cubic, "█".repeat(cubic as usize));
    }
}
