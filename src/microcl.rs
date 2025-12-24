use crate::prelude::KeyFrameFunction;

pub struct UClassList<const N: usize>(pub [(&'static str, KeyFrameFunction); N]);
