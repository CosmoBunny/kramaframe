use kramaframe::{
    BTframelist, KramaFrame,
    keyframe::KeyFrameFunction,
    keylist::{KeyList, TRES16Bits},
};

fn main() {
    let mut kramaframe: KramaFrame<_, BTframelist<TRES16Bits, i32>> = KramaFrame::default();

    kramaframe.extend_iter_classlist([
        ("linear", KeyFrameFunction::Linear),
        ("easein", KeyFrameFunction::EaseIn),
        ("easeout", KeyFrameFunction::EaseOut),
        ("easeinout", KeyFrameFunction::EaseInOut),
        (
            "cubic",
            KeyFrameFunction::new_cubic_bezier_f32(0., 1.26, 1., -0.79),
        ),
        ("step", KeyFrameFunction::Steps(5)),
    ]);

    kramaframe.framelist.extend([
        ("linear", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("easein", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("easeout", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("easeinout", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("easeinout", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("cubic", KeyList::new(1, TRES16Bits::from_millis(1000))),
        ("step", KeyList::new(1, TRES16Bits::from_millis(1000))),
    ]);

    // Linear
    for _ in 0..=60 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 60));
        let linear =
            kramaframe.animate_by_closure_range("linear", 1, |x| !x, |ongoing| !ongoing, 0..90u32);
        println!("linear : {} : {}", linear, "█".repeat(linear as usize));
    }
    // EaseIn
    for _ in 0..=90 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 90));
        let easein =
            kramaframe.animate_by_closure_range("easein", 1, |x| !x, |ongoing| !ongoing, 0..90u32);
        println!("easein : {} : {}", easein, "█".repeat(easein as usize));
    }
    // EaseOut
    for _ in 0..=90 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 90));
        let easeout =
            kramaframe.animate_by_closure_range("easeout", 1, |x| !x, |ongoing| !ongoing, 0..90u32);
        println!("easeout : {} : {}", easeout, "█".repeat(easeout as usize));
    }
    // EaseInOut
    for _ in 0..=90 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 90));
        let easeinout = kramaframe.animate_by_closure_range(
            "easeinout",
            1,
            |x| !x,
            |ongoing| !ongoing,
            0..90u32,
        );
        println!(
            "easeinout : {} : {}",
            easeinout,
            "█".repeat(easeinout as usize)
        );
    }
    // Cubic
    for _ in 0..=90 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 90));
        let cubic =
            kramaframe.animate_by_closure_range("cubic", 1, |x| !x, |ongoing| !ongoing, 0..90u32);
        println!("cubic : {} : {}", cubic, "█".repeat(cubic as usize));
    }
    // Steps
    for _ in 0..=90 {
        kramaframe.update_progress(TRES16Bits::from_millis(1000 / 90));
        let steps =
            kramaframe.animate_by_closure_range("step", 1, |x| !x, |ongoing| !ongoing, 0..90u32);
        println!("steps : {} : {}", steps, "█".repeat(steps as usize));
    }
}
