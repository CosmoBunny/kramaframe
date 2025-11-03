use std::{
    collections::BTreeMap,
    ops::{Add, Mul, Neg, Range, RangeInclusive, Sub},
};

use crate::keyframe::KeyFrameFunction;

macro_rules! interpolate_range_common {
    // simple Range<T>
    ($self_val:expr, $range_val:expr, $t_val:expr) => {{
        let range_length = $range_val.end - $range_val.start;
        if $self_val.is_reverse() {
            $range_val.end - range_length * $t_val
        } else {
            $range_val.start + range_length * $t_val
        }
    }};
    // simple RangeInclusive<T>
    ($self_val:expr, =, $range_val:expr, $t_val:expr) => {{
        let range_length = *($range_val.end()) - *($range_val.start());
        if $self_val.is_reverse() {
            *$range_val.end() - range_length * $t_val
        } else {
            *$range_val.start() + range_length * $t_val
        }
    }};
    // for u8, u16, u32, u64, u128, i8, i16, i32, i64, i128
    // simple Range<T> with `as $t conversion`
    ($self_val:expr, $range_val:expr, $t_val:expr, as $t:ty) => {{
        let range_reverse = $range_val.end < $range_val.start;
        let range_length = if range_reverse {
            $range_val.start - $range_val.end
        } else {
            $range_val.end - $range_val.start
        };

        if $self_val.is_reverse() {
            if range_reverse {
                $range_val.start - (range_length as f32 * $t_val) as $t
            } else {
                $range_val.end - (range_length as f32 * $t_val) as $t
            }
        } else {
            if range_reverse {
                $range_val.end + (range_length as f32 * $t_val) as $t
            } else {
                $range_val.start + (range_length as f32 * $t_val) as $t
            }
        }
    }};
    // simple RangeInclusive<T>
    ($self_val:expr, =, $range_val:expr, $t_val:expr, as $t:ty) => {{
        let range_reverse = *$range_val.end() < *$range_val.start();
        let range_length = if range_reverse {
            *$range_val.start() - *$range_val.end()
        } else {
            *$range_val.end() - *$range_val.start()
        };
        if $self_val.is_reverse() {
            if range_reverse {
                *$range_val.start() - (range_length as f32 * $t_val) as $t
            } else {
                *$range_val.end() - (range_length as f32 * $t_val) as $t
            }
        } else {
            if range_reverse {
                *$range_val.end() + (range_length as f32 * $t_val) as $t
            } else {
                *$range_val.start() + (range_length as f32 * $t_val) as $t
            }
        }
    }};
}

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
    pub fn get(&self, id: u32) -> Option<&ProgressList<TRES, PRES>> {
        self.progresses.get(&id)
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
    pub fn reverse_start(&mut self) {
        self.progress = PRES::max();
        self.reverse();
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

    fn progress_for_x(&self, progress: f32) -> f32 {
        let x = if self.is_reverse() {
            1.0 - progress.abs()
        } else {
            progress.abs()
        };
        x
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
            // Reverse and forward are the same.
            KeyFrameFunction::Linear => range.start + range_length * progress.abs(),
            KeyFrameFunction::Ease => {
                let x = if self.is_reverse() {
                    1.0 - progress.abs()
                } else {
                    progress.abs()
                };

                let eased_progress = cubic_bezier_y(x, 0.25, 0.1, 0.25, 1.0);
                interpolate_range_common!(self, range, eased_progress)
            }
            KeyFrameFunction::EaseIn => {
                let x = self.progress_for_x(progress);

                let eased_progress = cubic_bezier_y(x, 0.42, 0.0, 1.0, 1.0);
                interpolate_range_common!(self, range, eased_progress)
            }
            KeyFrameFunction::EaseOut => {
                let x = self.progress_for_x(progress);

                let eased_progress = cubic_bezier_y(x, 0.0, 0.0, 0.58, 1.0);
                interpolate_range_common!(self, range, eased_progress)
            }
            KeyFrameFunction::EaseInOut => {
                let x = self.progress_for_x(progress);

                let eased_progress = cubic_bezier_y(x, 0.42, 0.0, 1.0, 1.0);
                interpolate_range_common!(self, range, eased_progress)
            }
            KeyFrameFunction::CubicBezier(parameter) => {
                let x = self.progress_for_x(progress);
                let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                let cubic_bezier = cubic_bezier_y(x, x1, y1, x2, y2);
                interpolate_range_common!(self, range, cubic_bezier)
            }
            KeyFrameFunction::Quadratic => {
                let x = self.progress_for_x(progress);
                let quadratic = x.abs() * x.abs();
                interpolate_range_common!(self, range, quadratic)
            }
            KeyFrameFunction::Steps(step) => {
                let x = self.progress_for_x(progress);
                let step = (*step as f32).max(1.0);

                let stepped_progress = ((x * step).floor() / step).min(1.0);

                interpolate_range_common!(self, range, stepped_progress)
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
                let x = self.progress_for_x(progress);
                let eased_progress = cubic_bezier_y(x.abs(), 0.25, 0.1, 0.25, 1.0);
                interpolate_range_common!(self, =, range, eased_progress)
            }
            KeyFrameFunction::EaseIn => {
                let x = self.progress_for_x(progress);
                let eased_progress = cubic_bezier_y(x.abs(), 0.42, 0.0, 1.0, 1.0);
                interpolate_range_common!(self, =, range, eased_progress)
            }
            KeyFrameFunction::EaseOut => {
                let x = self.progress_for_x(progress);
                let eased_progress = cubic_bezier_y(x.abs(), 0.0, 0.0, 0.58, 1.0);
                interpolate_range_common!(self, =, range, eased_progress)
            }
            KeyFrameFunction::EaseInOut => {
                let x = self.progress_for_x(progress);
                let eased_progress = cubic_bezier_y(x.abs(), 0.25, 0.1, 0.25, 1.0);
                interpolate_range_common!(self, =, range, eased_progress)
            }
            KeyFrameFunction::CubicBezier(parameter) => {
                let x = self.progress_for_x(progress);
                let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                let cubic_bezier = cubic_bezier_y(x, x1, y1, x2, y2);
                interpolate_range_common!(self, =, range, cubic_bezier)
            }
            KeyFrameFunction::Quadratic => {
                let x = self.progress_for_x(progress);
                let quadratic = x.abs() * x.abs();
                interpolate_range_common!(self, =, range, quadratic)
            }
            KeyFrameFunction::Steps(step) => {
                let step = (*step as f32).max(1.0);
                let x = self.progress_for_x(progress);
                let stepped_progress = ((x * step).floor() / step).min(1.0);

                interpolate_range_common!(self, =, range, stepped_progress)
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
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.25, 0.1, 0.25, 1.0);
                        interpolate_range_common!(self, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseIn => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.42, 0.0, 1.0, 1.0);
                        interpolate_range_common!(self, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseOut => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.0, 0.0, 0.58, 1.0);
                        interpolate_range_common!(self, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseInOut => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.42, 0.0, 0.58, 1.0);
                        interpolate_range_common!(self, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::CubicBezier(parameter) => {
                        let x = self.progress_for_x(progress);
                        let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                        let cubic_bezier = cubic_bezier_y(x.abs(), x1, y1, x2, y2);
                        interpolate_range_common!(self, range, cubic_bezier, as $t)
                    }
                    KeyFrameFunction::Quadratic => {
                        let x = self.progress_for_x(progress);
                        let quadratic = x.abs() * x.abs();
                        interpolate_range_common!(self, range, quadratic, as $t)
                    }
                    KeyFrameFunction::Steps(step) => {
                        let step = (*step as f32).max(1.0);
                        let x = self.progress_for_x(progress);

                        let stepped_progress = ((x * step).floor() / step).min(1.0);

                        interpolate_range_common!(self, range, stepped_progress, as $t)
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
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.25, 0.1, 0.25, 1.0);
                        interpolate_range_common!(self, =, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseIn => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.42, 0.0, 1.0, 1.0);
                        interpolate_range_common!(self, =, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseOut => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.0, 0.0, 0.58, 1.0);
                        interpolate_range_common!(self, =, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::EaseInOut => {
                        let x = self.progress_for_x(progress);
                        let eased_progress = cubic_bezier_y(x.abs(), 0.42, 0.0, 0.58, 1.0);
                        interpolate_range_common!(self, =, range, eased_progress, as $t)
                    }
                    KeyFrameFunction::CubicBezier(parameter) => {
                        let x = self.progress_for_x(progress);
                        let [x1, y1, x2, y2] = parameter.map(|x| x as f32 / 10000.);
                        let cubic_bezier = cubic_bezier_y(x.abs(), x1, y1, x2, y2);
                        interpolate_range_common!(self, =, range, cubic_bezier, as $t)
                    }
                    KeyFrameFunction::Quadratic => {
                        let x = self.progress_for_x(progress);
                        let quadratic = x.abs() * x.abs();
                        interpolate_range_common!(self, =, range, quadratic, as $t)
                    }
                    KeyFrameFunction::Steps(step) => {
                        let x = self.progress_for_x(progress);
                        let step = (*step as f32).max(1.0);

                        let stepped_progress = ((x * step).floor() / step).min(1.0);

                        interpolate_range_common!(self, =, range, stepped_progress, as $t)
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
        *self < 0.0 && *self >= -1.0
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
                ({ <$t>::MIN + 1 }..0).contains(self)
            }
            fn start_signal(&self) -> bool {
                *self == <$t>::MIN
            }
            fn to_f32(&self) -> f32 {
                if *self == Self::MIN {
                    0.0
                } else {
                    *self as f32 / <$t>::MAX as f32
                }
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

#[test]
fn test_progress_resolution() {
    assert_eq!(i8::MIN, -128);
    assert_eq!(i8::MAX, 127);
    assert_eq!(i16::MIN, -32768);
    assert_eq!(i16::MAX, 32767);
    assert_eq!(i32::MIN, -2147483648);
    assert_eq!(i32::MAX, 2147483647);
    assert_eq!(i64::MIN, -9223372036854775808);
    assert_eq!(i64::MAX, 9223372036854775807);
}

#[test]
fn test_reverse() {
    assert_eq!(true, (-505i32).is_reverse());
    assert_eq!(false, (-128i8).is_reverse());
    assert_eq!(false, (123i8).is_reverse());
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
