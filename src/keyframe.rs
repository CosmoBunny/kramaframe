#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyFrameFunction {
    #[default]
    Linear,
    Ease,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// very small datatype for make memory and performance efficient.
    /// i16 is ranging -32768 to 32767 which it'll assume as -3.2768 to 3.2767 which is enough for cubic bezier parameter.
    /// So, it's good step to avoid floating error.
    CubicBezier([i16; 4]),
    /// Simple Quadratic Keyframe function
    Quadratic,
    Steps(u16),
}

impl KeyFrameFunction {
    pub fn new_linear() -> Self {
        KeyFrameFunction::Linear
    }
    pub fn new_ease() -> Self {
        KeyFrameFunction::Ease
    }
    pub fn new_ease_in() -> Self {
        KeyFrameFunction::EaseIn
    }
    pub fn new_ease_out() -> Self {
        KeyFrameFunction::EaseOut
    }
    pub fn new_ease_in_out() -> Self {
        KeyFrameFunction::EaseInOut
    }
    pub fn new_cubic_bezier(a: i16, b: i16, c: i16, d: i16) -> Self {
        KeyFrameFunction::CubicBezier([a, b, c, d])
    }
    pub fn new_cubic_bezier_f32(a: f32, b: f32, c: f32, d: f32) -> Self {
        KeyFrameFunction::CubicBezier([
            (a * 10000.0) as i16,
            (b * 10000.0) as i16,
            (c * 10000.0) as i16,
            (d * 10000.0) as i16,
        ])
    }
    pub fn new_quadratic() -> Self {
        KeyFrameFunction::Quadratic
    }
    pub fn new_steps(steps: u16) -> Self {
        KeyFrameFunction::Steps(steps)
    }
}
