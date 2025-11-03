use kramaframe::{BTframelist, KramaFrame, keylist::TRES16Bits};

fn main() {
    let mut animation_instance: KramaFrame<_, BTframelist<_, i16>> = KramaFrame::default();
    animation_instance
        .classlist
        .insert("sample", kramaframe::prelude::KeyFrameFunction::EaseIn);
    animation_instance.insert_new_id("sample", 1, TRES16Bits::from_millis(600));
    animation_instance.restart_progress("sample", 1);

    for i in 0..=60 {
        animation_instance.update_progress(TRES16Bits::from_millis(16));
        let value = animation_instance.get_value_byrange_inclusive("sample", 1, 100..=10u32);
        println!("Value at frame {}: {}", i, value);
    }
}
