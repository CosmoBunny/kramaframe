use kramaframe::{BTclasslist, BTframelist, KramaFrame, keyframe::KeyFrameFunction};

fn main() {
    let mut kramaframe: KramaFrame<BTclasslist, BTframelist<_, i32>> = KramaFrame::default();
    kramaframe.extend_iter_classlist([("animation1", KeyFrameFunction::Linear)]);
    kramaframe.insert_new_id("animation1", 1, 1.0);
    kramaframe.restart_progress("animation1", 1);

    for _ in 0..=121 {
        kramaframe.update_progress(1. / 120.);
        println!(
            "progress= {}, value = {}",
            kramaframe.get_progress_f32("animation1", 1),
            kramaframe.get_value_byrange_inclusive("animation1", 1, 80f32..=100f32)
        );
    }

    kramaframe.reverse_animate("animation1", 1);

    for _ in 0..=121 {
        kramaframe.update_progress(1. / 120.);
        println!(
            "progress= {}, value = {}",
            kramaframe.get_progress_f32("animation1", 1),
            kramaframe.get_value_byrange_inclusive("animation1", 1, 80f32..=100f32)
        );
    }
}
