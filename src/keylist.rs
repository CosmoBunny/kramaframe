use std::{
    collections::BTreeMap,
    ops::{Add, Mul, Neg, Range, RangeInclusive, Sub},
};

use crate::keyframe::KeyFrameFunction;

/**
List of Progress is stored by ID. Each specific ID has a specific timing and different progress.

*/
pub struct KeyList<TRES: TimingResolution, PRES: ProgressResolution + Eq> {
    progresses: BTreeMap<u32, ProgressList<TRES, PRES>>,
}

impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> Default for KeyList<TRES, PRES> {
    fn default() -> Self {
        KeyList {
            progresses: BTreeMap::new(),
        }
    }
}

// From Array that create new KeyList.
// Example: [400,500,800,400] each array index become id and it's element become timing.
impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> From<Vec<TRES>>
    for KeyList<TRES, PRES>
{
    fn from(vec: Vec<TRES>) -> Self {
        let mut keylist = KeyList::default();
        for (id, time) in vec.into_iter().enumerate() {
            keylist.new_id(id as u32, time);
        }
        keylist
    }
}
// Slice also
impl<TRES: TimingResolution, PRES: ProgressResolution + Eq, const N: usize> From<[TRES; N]>
    for KeyList<TRES, PRES>
{
    fn from(slice: [TRES; N]) -> Self {
        let mut keylist = Self::default();
        for (id, time) in slice.into_iter().enumerate() {
            keylist.new_id(id as u32, time);
        }
        keylist
    }
}

impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> KeyList<TRES, PRES> {
    pub fn new(id: u32, time: TRES) -> Self {
        let mut keylist = KeyList::default();
        keylist.new_id(id, time);
        keylist
    }

    /// Create new with Iterator like [(3,1000), (4,2000)] (id, timing)
    pub fn new_iter(iter: impl Iterator<Item = (u32, TRES)>) -> Self {
        let mut keylist = KeyList::default();
        for (id, time) in iter {
            keylist.new_id(id, time);
        }
        keylist
    }

    /// replace and add new id same as new but replace existing iterator id. non-same id will stay.
    pub fn replace_id_iter(&mut self, iter: impl Iterator<Item = (u32, TRES)>) {
        for (id, time) in iter {
            if let Some(progress_list) = self.progresses.get_mut(&id) {
                progress_list.time = time;
            } else {
                self.new_id(id, time);
            }
        }
    }

    pub fn set_time(&mut self, id: u32, time: TRES) {
        if let Some(progress_list) = self.progresses.get_mut(&id) {
            progress_list.time = time;
        }
    }

    pub fn new_id(&mut self, id: u32, time: TRES) {
        self.progresses.insert(
            id,
            ProgressList {
                time,
                progress: PRES::zero(),
            },
        );
    }
    pub fn change_timing(&mut self, id: u32, new_timing: TRES) {
        if let Some(progress_list) = self.progresses.get_mut(&id) {
            progress_list.time = new_timing;
        }
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&u32, &mut ProgressList<TRES, PRES>)> {
        self.progresses
            .iter_mut()
            .map(|(id, progress)| (id, progress))
    }
    pub fn get_mut(&mut self, id: u32) -> Option<&mut ProgressList<TRES, PRES>> {
        self.progresses.get_mut(&id)
    }
    /// function to start animation.
    pub fn start_animation(&mut self, id: u32) {
        if let Some(progress_list) = self.progresses.get_mut(&id) {
            progress_list.restart();
        }
    }
    pub fn get_progress_f32(&mut self, id: u32) -> f32 {
        self.progresses
            .get_mut(&id)
            .map(|progress| progress.get_progress_f32())
            .unwrap_or(0.0)
    }
}

pub struct ProgressList<TRES: TimingResolution, PRES: ProgressResolution + Eq> {
    time: TRES,
    progress: PRES,
}

impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> ProgressList<TRES, PRES> {
    pub fn new(time: TRES, progress: PRES) -> Self {
        Self { time, progress }
    }
    // Existing function from ProgressResolution.
    pub fn get_progress(&self) -> PRES {
        self.progress.absolute()
    }
    pub fn set_progress(&mut self, new_progress: PRES) {
        self.progress = new_progress;
    }
    pub fn reverse(&mut self) {
        self.progress.reverse();
    }
    pub fn restart(&mut self) {
        self.progress.restart();
    }
    pub fn max(&mut self) {
        self.progress = PRES::max();
    }
    pub fn zero(&mut self) {
        self.progress = PRES::zero();
    }
    pub fn get_progress_f32(&self) -> f32 {
        self.progress.to_f32()
    }
    pub fn get_time_f32(&self) -> f32 {
        self.time.to_sec()
    }
    pub fn is_animating(&self) -> bool {
        self.progress != PRES::max() && self.progress != PRES::zero()
    }

    pub fn is_reverse(&self) -> bool {
        self.progress.is_reverse()
    }

    pub fn update_progress(&mut self, delta_time: &TRES) {
        // new_pos = current_pos + (delta_time / animation_duration)
        // update progress if progress is Max or Zero
        if self.progress == PRES::max() || self.progress == PRES::zero() {
            return;
        }

        let increment = delta_time.to_sec() / self.time.to_sec();

        // Animation just started (via restart())
        if self.progress.start_signal() {
            let new_progress = increment.min(1.0);
            self.set_progress(PRES::from_f32(new_progress));
        } else {
            let current_progress_f32 = self.progress.to_f32();
            if self.progress.is_reverse() {
                // Progress is negative, going from -1 to 0.
                let new_progress = (current_progress_f32 + increment).min(0.0);
                self.set_progress(PRES::from_f32(new_progress));
            } else {
                // Progress is positive, going from 0 to 1.
                let new_progress = (current_progress_f32 + increment).min(1.0);
                self.set_progress(PRES::from_f32(new_progress));
            }
        }
    }
}

pub trait GetValueByGeneric<T> {
    fn get_generic_byrange(&self, range: Range<T>, keyframe: &KeyFrameFunction) -> T;
    fn get_generic_byrangeinclusive(
        &self,
        range: RangeInclusive<T>,
        keyframe: &KeyFrameFunction,
    ) -> T;
}

impl<T, TRES: TimingResolution, PRES: ProgressResolution + Eq> GetValueByGeneric<T>
    for ProgressList<TRES, PRES>
where
    T: Sized + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
{
    fn get_generic_byrange(&self, range: Range<T>, keyframe: &KeyFrameFunction) -> T {
        let range_length = range.end - range.start;
        let progress = self.progress.to_f32();
        match keyframe {
            KeyFrameFunction::Linear => range.start + range_length * progress.abs(),
            KeyFrameFunction::Ease => {
                // Standard ease, equivalent to cubic-bezier(0.25, 0.1, 0.25, 1.0)
                let eased_progress = cubic_bezier_y(progress.abs(), 0.25, 0.1, 0.25, 1.0);
                range.start + range_length * eased_progress
            }
            KeyFrameFunction::EaseIn => range.start + range_length * progress.abs().powf(2.0),
            KeyFrameFunction::EaseOut => {
                range.start + range_length * (1.0 - (1.0 - progress.abs()).powf(2.0))
            }
            KeyFrameFunction::EaseInOut => {
                range.start
                    + range_length
                        * (3.0 * progress.abs().powf(2.0) - 2.0 * progress.abs().powf(3.0))
            }
            KeyFrameFunction::CubicBezier(parameter) => {
                let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                let cubic_bezier = cubic_bezier_y(progress.abs(), x1, y1, x2, y2);
                range.start + range_length * cubic_bezier
            }
            KeyFrameFunction::Quadratic => {
                let quadratic = progress.abs() * progress.abs();
                range.start + range_length * quadratic
            }
            KeyFrameFunction::Steps(step) => {
                let step = (*step as f32).max(1.0);
                let progress = progress.abs();

                let stepped_progress = ((progress * step).floor() / step).min(1.0);

                range.start + range_length * stepped_progress
            }
        }
    }
    /// Get the value of the keyframe at the given range and keyframe function.
    fn get_generic_byrangeinclusive(
        &self,
        range: RangeInclusive<T>,
        keyframe: &KeyFrameFunction,
    ) -> T {
        let range_length = *range.end() - *range.start();
        let progress = self.progress.to_f32();
        match keyframe {
            KeyFrameFunction::Linear => *range.start() + range_length * progress.abs(),
            KeyFrameFunction::Ease => {
                // Standard ease, equivalent to cubic-bezier(0.25, 0.1, 0.25, 1.0)
                let eased_progress = cubic_bezier_y(progress.abs(), 0.25, 0.1, 0.25, 1.0);
                *range.start() + range_length * eased_progress
            }
            KeyFrameFunction::EaseIn => *range.start() + range_length * progress.abs().powf(2.0),
            KeyFrameFunction::EaseOut => {
                *range.start() + range_length * (1.0 - (1.0 - progress.abs()).powf(2.0))
            }
            KeyFrameFunction::EaseInOut => {
                *range.start()
                    + range_length
                        * (3.0 * progress.abs().powf(2.0) - 2.0 * progress.abs().powf(3.0))
            }
            KeyFrameFunction::CubicBezier(parameter) => {
                let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                let cubic_bezier = cubic_bezier_y(progress.abs(), x1, y1, x2, y2);
                *range.start() + range_length * cubic_bezier
            }
            KeyFrameFunction::Quadratic => {
                let quadratic = progress.abs() * progress.abs();
                *range.start() + range_length * quadratic
            }
            KeyFrameFunction::Steps(step) => {
                let step = (*step as f32).max(1.0);
                let progress = progress.abs();

                let stepped_progress = ((progress * step).floor() / step).min(1.0);

                *range.start() + range_length * stepped_progress
            }
        }
    }
}

pub trait GetValueByRange<T> {
    fn get_value_byrange(&self, range: Range<T>, keyframe: &KeyFrameFunction) -> T;
    fn get_value_byrangeinclusive(
        &self,
        range: RangeInclusive<T>,
        keyframe: &KeyFrameFunction,
    ) -> T;
}

macro_rules! impl_get_value_by_range {
    ($t:ty) => {
        impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> GetValueByRange<$t>
            for ProgressList<TRES, PRES>
        {
            fn get_value_byrange(&self, range: Range<$t>, keyframe: &KeyFrameFunction) -> $t {
                let range_length = range.end - range.start;
                let progress = self.progress.to_f32();
                match keyframe {
                    KeyFrameFunction::Linear => {
                        range.start + ((range_length as f32) * progress.abs()) as $t
                    }
                    KeyFrameFunction::Ease => {
                        let eased_progress = cubic_bezier_y(progress.abs(), 0.25, 0.1, 0.25, 1.0);
                        range.start + (range_length as f32 * eased_progress) as $t
                    }
                    KeyFrameFunction::EaseIn => {
                        range.start + ((range_length as f32) * progress.abs().powf(2.0)) as $t
                    }
                    KeyFrameFunction::EaseOut => {
                        range.start
                            + ((range_length as f32) * (1.0 - (1.0 - progress.abs()).powf(2.0)))
                                as $t
                    }
                    KeyFrameFunction::EaseInOut => {
                        range.start
                            + ((range_length as f32)
                                * (3.0 * progress.abs().powf(2.0) - 2.0 * progress.abs().powf(3.0)))
                                as $t
                    }
                    KeyFrameFunction::CubicBezier(parameter) => {
                        let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                        let cubic_bezier = cubic_bezier_y(progress.abs(), x1, y1, x2, y2);
                        range.start + ((range_length as f32) * cubic_bezier) as $t
                    }
                    KeyFrameFunction::Quadratic => {
                        let quadratic = progress.abs() * progress.abs();
                        range.start + (range_length as f32 * quadratic) as $t
                    }
                    KeyFrameFunction::Steps(step) => {
                        let step = (*step as f32).max(1.0);
                        let progress = progress.abs();

                        let stepped_progress = ((progress * step).floor() / step).min(1.0);

                        range.start + (range_length as f32 * stepped_progress) as $t
                    }
                }
            }
            fn get_value_byrangeinclusive(
                &self,
                range: RangeInclusive<$t>,
                keyframe: &KeyFrameFunction,
            ) -> $t {
                let range_length = *range.end() - *range.start();
                let progress = self.progress.to_f32();
                match keyframe {
                    KeyFrameFunction::Linear => {
                        *range.start() + ((range_length as f32) * progress.abs()) as $t
                    }
                    KeyFrameFunction::Ease => {
                        let eased_progress = cubic_bezier_y(progress.abs(), 0.25, 0.1, 0.25, 1.0);
                        *range.start() + (range_length as f32 * eased_progress) as $t
                    }
                    KeyFrameFunction::EaseIn => {
                        *range.start() + ((range_length as f32) * progress.abs().powf(2.0)) as $t
                    }
                    KeyFrameFunction::EaseOut => {
                        *range.start()
                            + ((range_length as f32) * (1.0 - (1.0 - progress.abs()).powf(2.0)))
                                as $t
                    }
                    KeyFrameFunction::EaseInOut => {
                        *range.start()
                            + ((range_length as f32)
                                * (3.0 * progress.abs().powf(2.0) - 2.0 * progress.abs().powf(3.0)))
                                as $t
                    }
                    KeyFrameFunction::CubicBezier(parameter) => {
                        let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                        let cubic_bezier = cubic_bezier_y(progress.abs(), x1, y1, x2, y2);
                        range.start() + ((range_length as f32) * cubic_bezier) as $t
                    }
                    KeyFrameFunction::Quadratic => {
                        let quadratic = progress.abs() * progress.abs();
                        *range.start() + (range_length as f32 * quadratic) as $t
                    }
                    KeyFrameFunction::Steps(step) => {
                        let step = (*step as f32).max(1.0);
                        let progress = progress.abs();

                        let stepped_progress = ((progress * step).floor() / step).min(1.0);

                        *range.start() + (range_length as f32 * stepped_progress) as $t
                    }
                }
            }
        }
    };
}

impl_get_value_by_range!(i8);
impl_get_value_by_range!(i16);
impl_get_value_by_range!(i32);
impl_get_value_by_range!(i64);
impl_get_value_by_range!(i128);
impl_get_value_by_range!(u8);
impl_get_value_by_range!(u16);
impl_get_value_by_range!(u32);
impl_get_value_by_range!(u64);
impl_get_value_by_range!(u128);
impl_get_value_by_range!(f32);
impl_get_value_by_range!(f64);

/**
Timing Resolution is the trait that defines the resolution of time. example u16 is ranging from 0 to 65535.
So, it's value would be milliseconds. And now time range becomes 0 to 65.535 seconds.
similarly u32 is ranging from 0 to 4294967295. but here's the twist, it's value is so huge that it can represent time up to 49.7 days.
who need 49.7 days to animate? So, it's value becomes microseconds. And then range becomes 0 to 42.9 seconds.
f32 is available but not recommended for time resolution as it has a limited precision.
*/
pub trait TimingResolution {
    fn to_sec(&self) -> f32;
    fn from_sec(sec: f32) -> Self;
}

pub struct TRES16Bits(pub u16);
impl TimingResolution for TRES16Bits {
    fn to_sec(&self) -> f32 {
        self.0 as f32 / 1000.0
    }
    fn from_sec(sec: f32) -> Self {
        Self((sec * 1000.0) as u16)
    }
}

impl TRES16Bits {
    pub fn from_millis(millis: u16) -> Self {
        Self(millis)
    }
    pub fn from_sec(sec: f32) -> Self {
        Self((sec * 1000.0) as u16)
    }
}

impl TimingResolution for u16 {
    fn to_sec(&self) -> f32 {
        *self as f32 / 1000.0
    }
    fn from_sec(sec: f32) -> Self {
        (sec * 1000.0) as u16
    }
}

pub struct TRES32Bits(pub u32);
impl TimingResolution for TRES32Bits {
    fn to_sec(&self) -> f32 {
        self.0 as f32 / 1000000.0
    }
    fn from_sec(sec: f32) -> Self {
        Self((sec * 1000000.0) as u32)
    }
}

impl TRES32Bits {
    pub fn from_millis(millis: u32) -> Self {
        Self(millis * 1000)
    }
    pub fn from_micros(micros: u32) -> Self {
        Self(micros)
    }
    pub fn from_seconds(seconds: f32) -> Self {
        Self((seconds * 1000000.0) as u32)
    }
}

impl TimingResolution for u32 {
    fn to_sec(&self) -> f32 {
        *self as f32 / 1000000.0
    }
    fn from_sec(sec: f32) -> Self {
        (sec * 1000000.0) as u32
    }
}

impl TimingResolution for f32 {
    fn to_sec(&self) -> f32 {
        *self
    }
    fn from_sec(sec: f32) -> Self {
        sec
    }
}

/**
Using f32 for progress (0.0 to 1.0) is inefficient for large-scale systems.
While switching to integers like i8 (using its range, e.g., 127 steps) saves memory, it introduces significant quantization errors.
For example, a 1.5-second animation at 60FPS requires a per-frame progress step of 127 steps / (1.5s * 60fps) \approx 1.41.
Since i8 can only store integers, rounding this to 1 discards 0.41, causing a 41% error (0.41 / 1.0) on that frame's increment,
which accumulates and leads to jerky or inaccurate animations.
Maximum animation error = (0.5/integer) * 100 %

## Example for i16:
- each frame takes 564 resolution steps.
- animation error = (0.0777778/564) * 100%  = 0.0213675 %
- maximum animation error = (0.5/564) * 100% = 0.1373626 %

## Example for i32:
- each frame takes 23860929.41 resolution steps.
- animation error = (0.41/23860929) * 100%  = 0.0000017 %
- maximum animation error = (0.5/23860929) * 100% = 0.0000021 %

Reversing progress is simple just negative the progress value. but progress value is still absolute for animation.
negative value is used for reverse animation.
if the progress value is maximum example 128 for i8. It means animation restart to Zero.
*/

pub trait ProgressResolution {
    fn absolute(&self) -> Self;
    fn reverse(&mut self);
    /// restart means set progress at specific value that represents the start signal. AKA forward/start function.
    /// If restart is not called then it's value stay in 0 or MIN-1 after animation and wait for restart signal.
    /// progress will goes MIN-1 (replace to zero for animation) to MAX.
    fn restart(&mut self);
    /// true for forward animation. false for reverse animation.
    fn is_reverse(&self) -> bool;

    fn start_signal(&self) -> bool;
    fn to_f32(&self) -> f32;
    fn from_f32(value: f32) -> Self;

    /// AKA Minimum absolute progress value.
    fn zero() -> Self;
    /// Maximum absolute progress value.
    fn max() -> Self;
}

impl ProgressResolution for f32 {
    fn absolute(&self) -> Self {
        self.abs()
    }
    fn restart(&mut self) {
        *self = -1.1
    }
    fn reverse(&mut self) {
        *self = self.neg();
    }
    fn start_signal(&self) -> bool {
        *self == -1.1
    }
    fn to_f32(&self) -> f32 {
        *self
    }
    fn from_f32(value: f32) -> Self {
        value
    }
    fn is_reverse(&self) -> bool {
        (-1f32..=0f32).contains(self)
    }
    fn zero() -> Self {
        0.0
    }
    fn max() -> Self {
        1.0
    }
}

macro_rules! impl_progress_resolution {
    ($t:ty) => {
        impl ProgressResolution for $t {
            fn absolute(&self) -> Self {
                self.saturating_abs()
            }
            fn reverse(&mut self) {
                *self = -*self;
            }
            fn restart(&mut self) {
                *self = <$t>::MIN;
            }
            fn is_reverse(&self) -> bool {
                (<$t>::MIN..=0).contains(self)
            }
            fn start_signal(&self) -> bool {
                *self == <$t>::MIN
            }
            fn to_f32(&self) -> f32 {
                *self as f32 / <$t>::MAX as f32
            }
            fn from_f32(value: f32) -> Self {
                (value * <$t>::MAX as f32).round() as $t
            }
            fn zero() -> Self {
                0
            }
            fn max() -> Self {
                <$t>::MAX
            }
        }
    };
}

impl_progress_resolution!(i8);
impl_progress_resolution!(i16);
impl_progress_resolution!(i32);
impl_progress_resolution!(i64);

fn cubic_bezier_y(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    // Solve for t in x(t) = x using Newton-Raphson
    let mut t = 0.5;
    for _ in 0..20 {
        let xt = 3.0 * x1 * t * (1.0 - t).powi(2) + 3.0 * x2 * t.powi(2) * (1.0 - t) + t.powi(3);
        let dxt = 3.0 * (1.0 - t).powi(2) * x1
            + 6.0 * t * (1.0 - t) * (x2 - x1)
            + 3.0 * t.powi(2) * (1.0 - x2);
        t -= (xt - x) / dxt.max(1e-6);
        t = t.clamp(0.0, 1.0);
    }
    3.0 * y1 * t * (1.0 - t).powi(2) + 3.0 * y2 * t.powi(2) * (1.0 - t) + t.powi(3)
}
