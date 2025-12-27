#[test]
fn test_macros() {
    use crate::btkramaframe;
    use crate::keylist::TRES16Bits;
    use crate::ukramaframe;

    let _ui_1 = ukramaframe!(<TRES16Bits, i16> "button" EaseIn [1,2,3,4,5,6] 1 s);
    let _ui_2 = ukramaframe!(<TRES16Bits, i16>
        "button" EaseIn [1,2,3,4,5,6] 1 s;
        "menu" EaseIn [1,2,3,4,5,6] 1 s;
    );
    let _ui_3 = ukramaframe!(<TRES16Bits, i16>
        "button" EaseIn [1,2,3,4,5,6] 1 s;
        "menu" EaseIn [1,2,3,4,5,6] 1 s;
        "dropdown" EaseIn [1,2,3,4,5,6] 1 s;
    );

    let _u1_big = btkramaframe!(
        <u32, i32>
        "button" EaseIn [1,3,4] 2.0 s;
    );
    assert!(true);
}
