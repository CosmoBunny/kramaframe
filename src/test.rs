#[test]
fn test_macros() {
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
    assert!(true);
}
