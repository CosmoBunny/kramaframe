#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "std"), feature = "alloc"))]
extern crate alloc;

#[cfg(all(not(feature = "std"), feature = "alloc"))]
pub use alloc::collections::BTreeMap;

#[cfg(feature = "std")]
pub use std::collections::BTreeMap;

use core::ops::RangeBounds;
use core::ops::{Range, RangeInclusive};

use crate::{
    keyframe::KeyFrameFunction,
    keylist::{GetValueByGeneric, GetValueByRange, ProgressResolution, TimingResolution},
    microcl::UClassList,
    microfl::UFrameList,
};

#[cfg(any(feature = "std", feature = "alloc"))]
use crate::keylist::KeyList;

// For non-alloc and no_std
pub mod microcl;
pub mod microfl;

pub mod test;
pub mod prelude {
    pub use crate::keyframe::KeyFrameFunction;
    #[cfg(any(feature = "std", feature = "alloc"))]
    pub use crate::keylist::KeyList;
    pub use crate::keylist::{
        GetValueByGeneric, GetValueByRange, ProgressResolution, TimingResolution,
    };
}

/// Provides the keyframe functions used for animation easing.
pub mod keyframe;
/// Provides the core data structures for managing lists of keyframes and their progress.
pub mod keylist;

/// The main animation controller.
///
/// `KramaFrame` manages a collection of animation "classes" and their corresponding animation "frames".
/// A class defines an animation behavior (e.g., easing function), while a frame represents a specific
/// instance of an animation with its own timing and progress.
///
/// - `CL`: The type for the class list, typically a map from a class name to a `KeyFrameFunction`.
/// - `FL`: The type for the frame list, typically a map from a class name to a `KeyList`.
pub struct KramaFrame<CL, FL> {
    /// A list of animation classes, mapping class names to keyframe functions (e.g., "linear", "ease-in").
    pub classlist: CL,
    /// A list of animation frames, mapping class names to `KeyList`s, which track individual animation instances.
    pub framelist: FL,
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<TRES: TimingResolution + Clone, PRES: ProgressResolution + Eq> Default
    for KramaFrame<BTreeMap<&'static str, KeyFrameFunction>, BTframelist<TRES, PRES>>
{
    /// Creates a new, empty `KramaFrame` with default BTreeMap-based storage.
    fn default() -> Self {
        KramaFrame {
            classlist: BTreeMap::new(),
            framelist: BTreeMap::new(),
        }
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<TRES: TimingResolution + Clone, PRES: ProgressResolution + Eq>
    KramaFrame<BTclasslist, BTreeMap<&'static str, KeyList<TRES, PRES>>>
{
    /// Extends the class list with new definitions or replaces existing ones.
    ///
    /// # Example
    ///
    /// ```ignore
    /// krama.extend_iter_classlist([
    ///     ("button1", KeyFrameFunction::Linear),
    ///     ("fade_in", KeyFrameFunction::EaseIn),
    /// ]);
    /// ```
    pub fn extend_iter_classlist<const N: usize>(
        &mut self,
        classlist: [(&'static str, KeyFrameFunction); N],
    ) {
        for (classname, keyframe) in classlist {
            self.classlist.insert(classname, keyframe);
        }
    }

    /// Inserts a new class with a default `KeyFrameFunction`.
    ///
    /// If the class name already exists, its keyframe function will be updated to the default.
    pub fn insert_new_class(&mut self, classname: &'static str) {
        self.classlist
            .insert(classname, KeyFrameFunction::default());
    }

    /// Inserts a new animation instance (identified by `id`) for a given class.
    ///
    /// If the class name does not exist in the `framelist`, a new `KeyList` will be created for it.
    pub fn insert_new_id(&mut self, on_classname: &'static str, id: u32, time: TRES) {
        if let Some(keylist) = self.framelist.get_mut(on_classname) {
            keylist.new_id(id, time);
        } else {
            let mut keylist = KeyList::default();
            keylist.new_id(id, time);
            self.framelist.insert(on_classname, keylist);
        }
    }

    /// Changes the total duration (`new_timing`) for a specific animation instance.
    pub fn change_timing(&mut self, on_classname: &'static str, id: u32, new_timing: TRES) {
        if let Some(framelist) = self.framelist.get_mut(on_classname) {
            framelist.change_timing(id, new_timing);
        }
    }

    /// Updates the progress of all active animations.
    ///
    /// This function should be called in your application's main loop (e.g., once per frame).
    /// It iterates through all registered classes and updates the progress of their associated
    /// animation instances based on the elapsed time (`delta_time`).
    ///
    /// # Arguments
    ///
    /// * `delta_time`: The time elapsed since the last update, typically in seconds.
    pub fn update_progress(&mut self, delta_time: TRES) {
        for (classname, _) in self.classlist.iter() {
            // check the existing classname on framelist
            if let Some(keylist) = self.framelist.get_mut(classname) {
                for (_, progresslist) in keylist.iter_mut() {
                    // updating progress based on delta time and KeyFrameFunction
                    progresslist.update_progress(&delta_time);
                }
            }
            // Else remains unchange. So CPU load is freed from undefined classname
        }
    }

    /// Restarts the progress of a specific animation instance.
    ///
    /// This sets its internal timer back to zero.
    pub fn restart_progress(&mut self, classname: &'static str, id: u32) {
        if let Some(keylist) = self.framelist.get_mut(classname) {
            if let Some(progresslist) = keylist.get_mut(id) {
                progresslist.restart();
            }
        }
    }

    /// Gets the current progress of a specific animation instance as a value between 0.0 and 1.0.
    ///
    /// Returns `0.0` if the class name or id is not found.
    /// Note: This is the raw progress, not yet modified by a `KeyFrameFunction`.
    pub fn get_progress_f32(&self, classname: &'static str, id: u32) -> f32 {
        if let Some(keylist) = self.framelist.get(classname) {
            if let Some(progresslist) = keylist.get(id) {
                return progresslist.get_progress_f32();
            }
        }
        0.0
    }

    pub fn remove_classname(&mut self, classname: &'static str) {
        self.framelist.remove(classname);
        self.classlist.remove(classname);
    }

    pub fn replace_classname(&mut self, old_classname: &'static str, new_classname: &'static str) {
        // check if the old class name exists
        if let Some(keyframe) = self.classlist.remove(old_classname) {
            if let Some(frames) = self.framelist.remove(old_classname) {
                // insert the frames with the new class name
                self.framelist.insert(new_classname, frames);
            }
            self.classlist.insert(new_classname, keyframe);
        }
    }
    pub fn set_timing(&mut self, classname: &'static str, id: u32, timing: TRES) {
        if let Some(frames) = self.framelist.get_mut(classname) {
            frames.set_time(id, timing);
        }
    }

    pub fn get_timing(&self, classname: &'static str, id: u32) -> TRES {
        if let Some(frames) = self.framelist.get(classname) {
            if let Some(progresslist) = frames.get(id) {
                return progresslist.get_time();
            }
        }
        TRES::zero()
    }

    pub fn is_reversed(&self, classname: &'static str, id: u32) -> bool {
        if let Some(keylist) = self.framelist.get(classname) {
            if let Some(progresslist) = keylist.get(id) {
                return progresslist.is_reverse();
            }
        }
        false
    }

    pub fn is_any_animation_inprogress(&self) -> bool {
        for (classname, _) in &self.classlist {
            if let Some(keylist) = self.framelist.get(classname) {
                for (_, progress) in keylist.get_progresses() {
                    if progress.is_animating() {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Gets the current elapsed time of a specific animation instance in seconds.
    ///
    /// Returns `0.0` if the class name or id is not found.
    pub fn get_time_f32(&self, classname: &'static str, id: u32) -> f32 {
        if let Some(keylist) = self.framelist.get(classname) {
            if let Some(progresslist) = keylist.get(id) {
                return progresslist.get_time_f32();
            }
        }
        0.0
    }

    /// rangebounded interpolated to get value from range bound such as start..end, start..=end, ..end and ..=end
    /// but it return default value if range is start.., .., =.. and start=..
    pub fn from_range<T>(
        &self,
        on_classname: &'static str,
        id: u32,
        range: impl RangeBounds<T>,
    ) -> T
    where
        T: Clone + Default,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        match (range.start_bound(), range.end_bound()) {
            (core::ops::Bound::Included(start), core::ops::Bound::Included(end)) => {
                self.get_value_byrange_inclusive(on_classname, id, start.clone()..=end.clone())
            }
            (core::ops::Bound::Included(start), core::ops::Bound::Excluded(end)) => {
                self.get_value_byrange(on_classname, id, start.clone()..end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Included(end)) => {
                self.get_value_byrange_inclusive(on_classname, id, T::default()..=end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Excluded(end)) => {
                self.get_value_byrange(on_classname, id, T::default()..end.clone())
            }
            _ => T::default(),
        }
    }

    pub fn is_classname(&self, classname: &'static str) -> bool {
        self.classlist.contains_key(classname)
    }

    pub fn is_id(&self, on_classname: &'static str, id: u32) -> bool {
        self.framelist
            .get(on_classname)
            .map_or(false, |keylist| keylist.progresses.contains_key(&id))
    }

    /// Calculates and returns an interpolated value for an animation within a given `Range`.
    ///
    /// The interpolation is based on the animation's current progress and its class's `KeyFrameFunction`.
    /// This method is suitable for types that support the necessary arithmetic operations (`Add`, `Sub`, `Mul<f32>`).
    ///
    /// Returns the `range.start` value if the class name or id is not found.
    pub fn get_value_byrange<T>(&self, on_classname: &'static str, id: u32, range: Range<T>) -> T
    where
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get(on_classname) {
                if let Some(progresslist) = keylist.get(id) {
                    return progresslist.get_value_byrange(range, keyframe);
                }
            }
        }
        range.start
    }

    /// Calculates and returns an interpolated value for an animation within a given `RangeInclusive`.
    ///
    /// The interpolation is based on the animation's current progress and its class's `KeyFrameFunction`.
    /// This method is suitable for types that support the necessary arithmetic operations (`Add`, `Sub`, `Mul<f32>`).
    ///
    /// Returns the `range.start()` value if the class name or id is not found.
    pub fn get_value_byrange_inclusive<T>(
        &self,
        on_classname: &'static str,
        id: u32,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get(on_classname) {
                if let Some(progresslist) = keylist.get(id) {
                    return progresslist.get_value_byrangeinclusive(range, keyframe);
                }
            }
        }
        range.start().clone()
    }
    /// rangebounded interpolated to get value from range bound such as start..end, start..=end, ..end and ..=end
    /// but it return default value if range is start.., .., =.. and start=..
    pub fn from_range_generic<T>(
        &self,
        on_classname: &'static str,
        id: u32,
        range: impl RangeBounds<T>,
    ) -> T
    where
        T: Default + Clone + Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        match (range.start_bound(), range.end_bound()) {
            (core::ops::Bound::Included(start), core::ops::Bound::Included(end)) => self
                .get_generic_value_by_rangeinclusive(on_classname, id, start.clone()..=end.clone()),
            (core::ops::Bound::Included(start), core::ops::Bound::Excluded(end)) => {
                self.get_generic_byrange(on_classname, id, start.clone()..end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Included(end)) => self
                .get_generic_value_by_rangeinclusive(on_classname, id, T::default()..=end.clone()),
            (core::ops::Bound::Unbounded, core::ops::Bound::Excluded(end)) => {
                self.get_generic_byrange(on_classname, id, T::default()..end.clone())
            }
            _ => T::default(),
        }
    }

    /**
     Gets a value from a `Range` based on animation progress, for generic types.

     This method is intended for types that might not support arithmetic interpolation (e.g., enums).
     It relies on the `GetValueByGeneric` trait to determine the value based on progress.

     NOTE: Generic should be implemented with
     - Clone, Copy
     - Add<Output = Self>, Sub<Output = Self>
     - Mul<f32, Output = Self>

     ## Example of implementation
     ```rust
        use core::ops::{Add, Sub, Mul};
        #[derive(Clone, Copy)]
        struct Point {
            x: f32,
            y: f32,
        }

        impl Add for Point {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                Point {
                    x: self.x + other.x,
                    y: self.y + other.y,
                }
            }
        }

        impl Sub for Point {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                Point {
                    x: self.x - other.x,
                    y: self.y - other.y,
                }
            }
        }

        impl Mul<f32> for Point {
            type Output = Self;

            fn mul(self, scalar: f32) -> Self {
                Point {
                    x: self.x * scalar,
                    y: self.y * scalar,
                }
            }
        }
     ```
     Returns `range.start` if the class name or id is not found.
    */
    pub fn get_generic_byrange<T>(&self, on_classname: &'static str, id: u32, range: Range<T>) -> T
    where
        T: Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get(on_classname) {
                if let Some(progresslist) = keylist.get(id) {
                    return progresslist.get_generic_byrange(range, keyframe);
                }
            }
        }
        range.start
    }

    /**
     Gets a value from a `RangeInclusive` based on animation progress, for generic types.

     This method is intended for types that might not support arithmetic interpolation (e.g., enums).
     It relies on the `GetValueByGeneric` trait to determine the value based on progress.

     NOTE: Generic should be implemented with
     - Clone, Copy
     - Add<Output = Self>, Sub<Output = Self>
     - Mul<f32, Output = Self>

     ## Example of implementation
     ```rust
        use core::ops::{Add, Sub, Mul};
        #[derive(Clone, Copy)]
        struct Point {
            x: f32,
            y: f32,
        }

        impl Add for Point {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                Point {
                    x: self.x + other.x,
                    y: self.y + other.y,
                }
            }
        }

        impl Sub for Point {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                Point {
                    x: self.x - other.x,
                    y: self.y - other.y,
                }
            }
        }

        impl Mul<f32> for Point {
            type Output = Self;

            fn mul(self, scalar: f32) -> Self {
                Point {
                    x: self.x * scalar,
                    y: self.y * scalar,
                }
            }
        }
     ```
     Returns `range.start` if the class name or id is not found.
    */
    pub fn get_generic_value_by_rangeinclusive<T>(
        &self,
        on_classname: &'static str,
        id: u32,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get(on_classname) {
                if let Some(progresslist) = keylist.get(id) {
                    return progresslist.get_generic_byrangeinclusive(range, keyframe);
                }
            }
        }
        range.start().clone()
    }

    /// Reverses the direction of a specific animation instance.
    ///
    /// If it was playing forwards, it will now play backwards, and vice-versa.
    pub fn reverse_animate(&mut self, on_classname: &'static str, id: u32) {
        if let Some(keylist) = self.framelist.get_mut(on_classname) {
            if let Some(progresslist) = keylist.get_mut(id) {
                progresslist.reverse();
            }
        }
    }

    /// Reverses the direction of a specific animation instance and starts it.
    ///
    /// If it was playing forwards, it will now play backwards, and vice-versa.
    pub fn reverse_start(&mut self, on_classname: &'static str, id: u32) {
        if let Some(keylist) = self.framelist.get_mut(on_classname) {
            if let Some(progresslist) = keylist.get_mut(id) {
                progresslist.reverse_start();
            }
        }
    }

    /// Checks if a specific animation instance is currently playing.
    ///
    /// Returns `true` if the animation is playing, `false` otherwise.
    pub fn is_animating(&self, on_classname: &'static str, id: u32) -> bool {
        if let Some(keylist) = self.framelist.get(on_classname) {
            if let Some(progresslist) = keylist.get(id) {
                return progresslist.is_animating();
            }
        }
        false
    }

    /// A flexible method to control an animation's state using closures and retrieve its current value.
    ///
    /// This function allows you to dynamically control the start and direction of an animation
    /// while also getting its current interpolated value from a `Range`. It is designed for generic types
    /// that may not support arithmetic interpolation.
    ///
    /// # Arguments
    /// * `on_classname`: The name of the animation class.
    /// * `id`: The unique identifier for the animation instance.
    /// * `direction`: A closure that receives the current reverse state (`true` if playing backwards) and returns the desired direction (`true` for forward, `false` for reverse).
    /// * `start`: A closure that receives the current animating state (`true` if progress is between 0.0 and 1.0) and returns whether the animation should be restarted (`true` to restart).
    /// * `range`: The `Range<T>` of values to animate between.
    ///
    /// # Returns
    /// The calculated value for the current frame, before state changes from the closures are applied.
    pub fn animate_by_closure_rangegeneric<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        direction: fn(bool) -> bool,
        start: fn(bool) -> bool,
        range: Range<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    let previous = progresslist.get_generic_byrange(range, keyframe);
                    let restart_need = start(progresslist.is_animating());
                    let forward = direction(progresslist.is_reverse());

                    if progresslist.is_reverse() != !forward {
                        progresslist.reverse();
                    }

                    if restart_need {
                        progresslist.restart();
                    }
                    return previous;
                }
            }
        }
        range.start
    }

    /// A flexible method to control an animation's state using closures and retrieve its current value.
    ///
    /// This function allows you to dynamically control the start and direction of an animation
    /// while also getting its current interpolated value from a `Range`. It is designed for types
    /// that support arithmetic operations for smooth interpolation.
    ///
    /// # Arguments
    /// * `on_classname`: The name of the animation class.
    /// * `id`: The unique identifier for the animation instance.
    /// * `direction`: A closure that receives the current reverse state (`true` if playing backwards) and returns the desired direction (`true` for forward, `false` for reverse).
    /// * `start`: A closure that receives the current animating state (`true` if progress is between 0.0 and 1.0) and returns whether the animation should be restarted (`true` to restart).
    /// * `range`: The `Range<T>` of values to animate between.
    ///
    /// # Returns
    /// The calculated value for the current frame, before state changes from the closures are applied.
    pub fn animate_by_closure_range<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        direction: fn(bool) -> bool,
        start: fn(bool) -> bool,
        range: Range<T>,
    ) -> T
    where
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    let previous = progresslist.get_value_byrange(range, keyframe);
                    let restart_need = start(progresslist.is_animating());
                    let forward = direction(progresslist.is_reverse());

                    if progresslist.is_reverse() != !forward {
                        progresslist.reverse();
                    }

                    if restart_need {
                        progresslist.restart();
                    }
                    return previous;
                }
            }
        }
        range.start
    }
}

/// A type alias for a class list implemented with `BTreeMap`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub type BTclasslist = BTreeMap<&'static str, KeyFrameFunction>;
/// A type alias for a frame list implemented with `BTreeMap`.
#[cfg(any(feature = "std", feature = "alloc"))]
pub type BTframelist<TRES, PRES> = BTreeMap<&'static str, KeyList<TRES, PRES>>;

impl<'a, const N: usize, UN: Eq, TRES: TimingResolution + Clone, PRES: ProgressResolution + Eq>
    KramaFrame<UClassList<N>, UFrameList<'a, N, UN, TRES, PRES>>
{
    /// Changes the keyframe function (easing behavior) for a specific animation class.
    ///
    /// # Example
    /// ```ignore
    /// krama.change_keyframefunction("fade_in", KeyFrameFunction::EaseIn);
    /// ```
    pub fn change_keyframefunction(&mut self, classname: &'static str, new: KeyFrameFunction) {
        for (inclass, keyframe) in &mut self.classlist.0 {
            if *inclass == classname {
                *keyframe = new;
            }
        }
    }

    /// Updates the progress of all active animations in the frame list based on the provided delta time.
    ///
    /// # Example
    /// ```ignore
    /// krama.update_progress(TRES::from_sec(0.016));
    /// ```
    pub fn update_progress(&mut self, delta_time: TRES) {
        for item in self.framelist.0.iter_mut() {
            let ukeylists = &mut item.1;

            for ukeylist in ukeylists.iter_mut() {
                for (_, progreslist) in &mut *ukeylist.0 {
                    progreslist.update_progress(&delta_time);
                }
            }
        }
    }

    /// Restarts the progress of a specific animation instance identified by class name and ID.
    ///
    /// # Example
    /// ```ignore
    /// krama.restart_progress("button_click", 1);
    /// ```
    pub fn restart_progress(&mut self, classname: &'static str, id: UN) {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    for (inid, progresslist) in ukeylist.0.iter_mut() {
                        if *inid == id {
                            progresslist.restart();
                            break;
                        }
                    }
                }
                break;
            }
        }
    }

    /// Retrieves the current progress of an animation instance as a float between 0.0 and 1.0.
    ///
    /// # Example
    /// ```ignore
    /// let p = krama.get_progress_f32("spinner", 0);
    /// ```
    pub fn get_progress_f32(&mut self, classname: &'static str, id: UN) -> f32 {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    for (inid, progresslist) in ukeylist.0.iter_mut() {
                        if *inid == id {
                            return progresslist.get_progress_f32();
                        }
                    }
                }
                break;
            }
        }
        0.0
    }

    /// Updates the total duration (timing) for a specific animation instance.
    ///
    /// # Example
    /// ```ignore
    /// krama.set_timing("slow_move", 5, TRES::from_sec(10.0));
    /// ```
    pub fn set_timing(&mut self, classname: &'static str, id: UN, timing: TRES) {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    ukeylist.set_time(&id, timing.clone());
                }
                break;
            }
        }
    }

    /// Gets the configured total duration (timing) for a specific animation instance.
    ///
    /// # Example
    /// ```ignore
    /// let duration = krama.get_timing("move", 1);
    /// ```
    pub fn get_timing(&mut self, classname: &'static str, id: UN) -> TRES {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    return ukeylist.get_time(&id);
                }
                break;
            }
        }
        TRES::from_sec(0.0)
    }

    /// Checks if a specific animation instance is currently playing in reverse.
    ///
    /// # Example
    /// ```ignore
    /// if krama.is_reversed("door", 1) { println!("Closing..."); }
    /// ```
    pub fn is_reversed(&mut self, classname: &'static str, id: UN) -> bool {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    return ukeylist.is_reversed(&id);
                }
                break;
            }
        }
        false
    }

    /// Checks if any animation instance across all classes is currently in progress.
    ///
    /// # Example
    /// ```ignore
    /// if krama.is_any_animation_inprogress() {
    ///     request_next_frame();
    /// }
    /// ```
    pub fn is_any_animation_inprogress(&mut self) -> bool {
        for (_, ukeylists) in &mut self.framelist.0 {
            for ukeylist in ukeylists.iter_mut() {
                if ukeylist.is_any_animation_inprogress() {
                    return true;
                }
            }
        }
        false
    }

    /// Gets the current elapsed time in seconds for a specific animation instance.
    ///
    /// # Example
    /// ```ignore
    /// let seconds = krama.get_time_f32("timer", 1);
    /// ```
    pub fn get_time_f32(&mut self, classname: &'static str, id: UN) -> f32 {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    return ukeylist.get_time_f32(&id);
                }
                break;
            }
        }
        0.0
    }

    /// rangebounded interpolated to get value from range bound such as start..end, start..=end, ..end and ..=end
    /// but it return default value if range is start.., .., =.. and start=..
    pub fn from_range<T>(&self, on_classname: &'static str, id: UN, range: impl RangeBounds<T>) -> T
    where
        T: Clone + Default,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        match (range.start_bound(), range.end_bound()) {
            (core::ops::Bound::Included(start), core::ops::Bound::Included(end)) => {
                self.get_value_byrange_inclusive(on_classname, id, start.clone()..=end.clone())
            }
            (core::ops::Bound::Included(start), core::ops::Bound::Excluded(end)) => {
                self.get_value_byrange(on_classname, id, start.clone()..end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Included(end)) => {
                self.get_value_byrange_inclusive(on_classname, id, T::default()..=end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Excluded(end)) => {
                self.get_value_byrange(on_classname, id, T::default()..end.clone())
            }
            _ => T::default(),
        }
    }

    /// Calculates an interpolated value within a given `Range` based on an animation's progress.
    ///
    /// # Example
    /// ```ignore
    /// let x_pos = krama.get_value_byrange("move", 1, 0.0..500.0);
    /// ```
    pub fn get_value_byrange<T>(&self, classname: &'static str, id: UN, range: Range<T>) -> T
    where
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        let mut keyframe = KeyFrameFunction::Linear; // Start with default

        for (inclass, kf) in self.classlist.0.iter() {
            if *inclass == classname {
                keyframe = *kf; // Update value
                break; // Stop searching
            }
        }

        for (inclass, ukeylists) in &self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter() {
                    for (inid, progresslist) in &*ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_value_byrange(range, &keyframe);
                        }
                    }
                }
                break;
            }
        }
        range.start
    }

    /// Calculates an interpolated value within a given `RangeInclusive` based on an animation's progress.
    ///
    /// # Example
    /// ```ignore
    /// let alpha = krama.get_value_byrange_inclusive("fade", 1, 0.0..=1.0);
    /// ```
    pub fn get_value_byrange_inclusive<T>(
        &self,
        classname: &'static str,
        id: UN,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        let mut keyframe = KeyFrameFunction::Linear; // Start with default

        for (inclass, kf) in self.classlist.0.iter() {
            if *inclass == classname {
                keyframe = *kf; // Update value
                break; // Stop searching
            }
        }

        for (inclass, ukeylists) in &self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter() {
                    for (inid, progresslist) in &*ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_value_byrangeinclusive(range, &keyframe);
                        }
                    }
                }
                break;
            }
        }
        range.start().clone()
    }

    /// rangebounded interpolated to get generic value from range bound such as start..end, start..=end, ..end and ..=end
    /// but it return default value if range is start.., .., =.. and start=..
    pub fn from_range_generic<T>(
        &self,
        on_classname: &'static str,
        id: UN,
        range: impl RangeBounds<T>,
    ) -> T
    where
        T: Default + Clone + Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        match (range.start_bound(), range.end_bound()) {
            (core::ops::Bound::Included(start), core::ops::Bound::Included(end)) => self
                .get_generic_value_by_rangeinclusive(on_classname, id, start.clone()..=end.clone()),
            (core::ops::Bound::Included(start), core::ops::Bound::Excluded(end)) => {
                self.get_generic_byrange(on_classname, id, start.clone()..end.clone())
            }
            (core::ops::Bound::Unbounded, core::ops::Bound::Included(end)) => self
                .get_generic_value_by_rangeinclusive(on_classname, id, T::default()..=end.clone()),
            (core::ops::Bound::Unbounded, core::ops::Bound::Excluded(end)) => {
                self.get_generic_byrange(on_classname, id, T::default()..end.clone())
            }
            _ => T::default(),
        }
    }

    /// Gets an interpolated value for generic types using the `GetValueByGeneric` trait.
    ///
    /// # Example
    /// ```ignore
    /// let color = krama.get_generic_byrange("recolor", 1, RED..BLUE);
    /// ```
    pub fn get_generic_byrange<T>(&self, classname: &'static str, id: UN, range: Range<T>) -> T
    where
        T: Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        let mut keyframe = KeyFrameFunction::Linear; // Start with default

        for (inclass, kf) in self.classlist.0.iter() {
            if *inclass == classname {
                keyframe = *kf; // Update value
                break; // Stop searching
            }
        }

        for (inclass, ukeylists) in &self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter() {
                    for (inid, progresslist) in &*ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_generic_byrange(range, &keyframe);
                        }
                    }
                }
                break;
            }
        }
        range.start
    }

    /// Gets an interpolated value for generic types within a `RangeInclusive`.
    ///
    /// # Example
    /// ```ignore
    /// let pos = krama.get_generic_value_by_rangeinclusive("path", 1, start..=end);
    /// ```
    pub fn get_generic_value_by_rangeinclusive<T>(
        &self,
        classname: &'static str,
        id: UN,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        let mut keyframe = KeyFrameFunction::Linear; // Start with default

        for (inclass, kf) in self.classlist.0.iter() {
            if *inclass == classname {
                keyframe = *kf; // Update value
                break; // Stop searching
            }
        }

        for (inclass, ukeylists) in &self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter() {
                    for (inid, progresslist) in &*ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_generic_byrangeinclusive(range, &keyframe);
                        }
                    }
                }
                break;
            }
        }
        range.start().clone()
    }

    /// Reverses the playback direction of a specific animation instance.
    ///
    /// # Example
    /// ```ignore
    /// krama.reverse_animate("toggle", 1);
    /// ```
    pub fn reverse_animate(&mut self, classname: &'static str, id: UN) {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    ukeylist.reverse(&id);
                }
                break;
            }
        }
    }

    /// Reverses the playback direction and ensures the animation is active.
    ///
    /// # Example
    /// ```ignore
    /// krama.reverse_start("menu", 1);
    /// ```
    pub fn reverse_start(&mut self, classname: &'static str, id: UN) {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    ukeylist.reverse_start(&id);
                }
                break;
            }
        }
    }

    /// Checks if a specific animation instance is currently playing.
    ///
    /// # Example
    /// ```ignore
    /// if krama.is_animating("logo_spin", 1) { /* ... */ }
    /// ```
    pub fn is_animating(&mut self, classname: &'static str, id: UN) -> bool {
        for (inclass, ukeylists) in &mut self.framelist.0 {
            if *inclass == classname {
                for ukeylist in ukeylists.iter_mut() {
                    if ukeylist.is_animating(&id) {
                        return true;
                    }
                }
                break;
            }
        }
        false
    }
}

/// Creates a compile-time, zero-allocation `KramaFrame` instance using stack-allocated micro types.
///
/// This macro constructs a `KramaFrame<UClassList<'static>, UFrameList<'static, N, u32, TRES, PRES>>`
/// where all data resides on the stack (no heap allocations). It is intended for scenarios requiring
/// maximum performance and predictability, such as embedded systems or high-frequency animation updates.
///
/// The macro requires explicit specification of the timing and progress resolution types (`TRES` and `PRES`).
/// All keyframes are initialized with `progress: PRES::zero()` and the provided animation duration.
///
/// ### Syntax
///
/// ```text
/// ukramaframe!(
///     "class_name" EasingFunction [key_id1, key_id2, ..., key_idN] duration s ;
///     ...
/// );
/// ```
///
/// - `<TRES, PRES>`: Concrete types implementing `TimingResolution` and `ProgressResolution`, respectively.
/// - `"class_name"`: A string literal identifying the animation class (must be `'static`).
/// - `EasingFunction`: A variant of `KeyFrameFunction` (e.g., `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`).
/// - `[key_id1, key_id2, ...]`: A comma-separated list of `u32` literal key identifiers.
///   Trailing commas are permitted.
/// - `duration s`: Animation duration in seconds.
///   - Integer form: `4 s` → internally cast to `f32`.
///   - Floating-point form: `4.0 s` → used directly.
///
/// Multiple entries are separated by semicolons. A single entry may omit the trailing semicolon.
///
/// ### Examples
///
/// **Single entry (integer duration):**
///
/// ```ignore
/// use kramaframe::{ukramaframe, TRES16Bits};
///
/// let anim = ukramaframe!(<TRES16Bits, i16> "button" EaseIn [1, 2, 3, 4, 5, 6] 4 s);
/// ```
///
/// **Single entry (floating-point duration):**
///
/// ```ignore
/// use kramaframe::ukramaframe;
///
/// let anim = ukramaframe!(<TRES16Bits, i16> "button" EaseIn [1, 2, 3] 4.0 s);
/// ```
///
/// **Multiple entries (mixed durations):**
///
/// ```ignore
/// use kramaframe::ukramaframe;
///
/// let anim = ukramaframe!(<TRES16Bits, i16>
///     "menu"   EaseIn   [1, 2, 3]      2 s;
///     "button" EaseOut  [4, 5, 6, 7]  1.5 s;
///     "header" Linear   [8, 9]       3.0 s;
/// );
/// ```
///
/// No additional trait imports are required; the macro uses fully qualified paths for
/// `TimingResolution::from_sec` and `ProgressResolution::zero`.
///
/// This design ensures zero runtime overhead while preserving compile-time type safety and flexibility.
#[macro_export]
macro_rules! ukramaframe {
    // =========================================================================
    // BRANCH 1: 3 Generics <TRES, PRES, UN> (Explicit ID Type) - Multiple Entries
    // =========================================================================
    (<$TRES:ty, $PRES:ty, $UN:ty>
        $(
            $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ;
        )+
    ) => {{
        const N: usize = $crate::count!($( $class ),+);

        // Note: casting keys ($key as $UN) is crucial here
        const CLASSLIST_ARRAY: [(&'static str, $crate::keyframe::KeyFrameFunction); N] = [
            $( ($class, $crate::keyframe::KeyFrameFunction::$easing) ),+
        ];

        $crate::KramaFrame {
            classlist: $crate::microcl::UClassList(CLASSLIST_ARRAY),
            framelist: $crate::microfl::UFrameList([
                $(
                    (
                        $class,
                        &mut [ $crate::keylist::MicroKeyList(&mut [
                            $(
                                (
                                    $key as $UN,
                                    $crate::keylist::ProgressList::new(
                                        <$TRES>::from_sec($duration as f32),
                                        <$PRES as $crate::keylist::ProgressResolution>::zero(),
                                    )
                                ),
                            )*
                        ]) ]
                    ),
                )+
            ]),
        }
    }};

    // =========================================================================
    // BRANCH 2: 3 Generics <TRES, PRES, UN> (Explicit ID Type) - Single Entry
    // =========================================================================
    (<$TRES:ty, $PRES:ty, $UN:ty> $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ) => {{
        const CLASS: &'static str = $class;

        $crate::KramaFrame {
            classlist: $crate::microcl::UClassList([ (CLASS, $crate::keyframe::KeyFrameFunction::$easing) ]),
            framelist: $crate::microfl::UFrameList([(
                CLASS,
                &mut [ $crate::keylist::MicroKeyList(&mut [
                    $(
                        (
                            $key as $UN,
                            $crate::keylist::ProgressList::new(
                                <$TRES>::from_sec($duration as f32),
                                <$PRES as $crate::keylist::ProgressResolution>::zero(),
                            )
                        ),
                    )*
                ]) ]
            )]),
        }
    }};

    // =========================================================================
    // BRANCH 3: 2 Generics <TRES, PRES> (Inferred ID Type) - Multiple Entries
    // =========================================================================
    (<$TRES:ty, $PRES:ty>
        $(
            $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ;
        )+
    ) => {{
        const N: usize = $crate::count!($( $class ),+);

        const CLASSLIST_ARRAY: [(&'static str, $crate::keyframe::KeyFrameFunction); N] = [
            $( ($class, $crate::keyframe::KeyFrameFunction::$easing) ),+
        ];

        $crate::KramaFrame {
            classlist: $crate::microcl::UClassList(CLASSLIST_ARRAY),
            framelist: $crate::microfl::UFrameList([
                $(
                    (
                        $class,
                        &mut [ $crate::keylist::MicroKeyList(&mut [
                            $(
                                (
                                    $key, // Inferred type (usually i32/u32)
                                    $crate::keylist::ProgressList::new(
                                        <$TRES>::from_sec($duration as f32),
                                        <$PRES as $crate::keylist::ProgressResolution>::zero(),
                                    )
                                ),
                            )*
                        ]) ]
                    ),
                )+
            ]),
        }
    }};

    // =========================================================================
    // BRANCH 4: 2 Generics <TRES, PRES> (Inferred ID Type) - Single Entry
    // =========================================================================
    (<$TRES:ty, $PRES:ty> $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ) => {{
        const CLASS: &'static str = $class;

        $crate::KramaFrame {
            classlist: $crate::microcl::UClassList([ (CLASS, $crate::keyframe::KeyFrameFunction::$easing) ]),
            framelist: $crate::microfl::UFrameList([(
                CLASS,
                &mut [ $crate::keylist::MicroKeyList(&mut [
                    $(
                        (
                            $key,
                            $crate::keylist::ProgressList::new(
                                <$TRES>::from_sec($duration as f32),
                                <$PRES as $crate::keylist::ProgressResolution>::zero(),
                            )
                        ),
                    )*
                ]) ]
            )]),
        }
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! count {
    () => { 0 };
    ($head:tt $(, $tail:tt)*) => {
        1 + $crate::count!($($tail),*)
    };
}

/// Creates a `KramaFrame` instance initialized with `BTreeMap` storage (Heap allocated).
///
/// This macro provides a convenient, declarative syntax for initializing a `KramaFrame` that uses
/// `BTreeMap` (standard library). It mimics the syntax of `ukramaframe!` but constructs the maps dynamically.
///
/// Requires the `std` or `alloc` feature (implied by usage of `BTreeMap`).
///
/// ### Syntax
///
/// ```text
/// btkramaframe!(
///     <TRES, PRES>
///     "class_name" EasingFunction [key_id1, key_id2, ...] duration s ;
///     ...
/// );
/// ```
///
/// - `TRES`, `PRES`: Timing and Progress resolution types.
/// - IDs are strictly `u32` for `BTreeMap` implementation.
///
/// ### Example
///
/// ```ignore
/// let mut anim = btkramaframe!(
///     <u32, i32>
///     "fade" EaseIn [1, 2] 1.0 s;
///     "slide" Linear [10] 2.5 s;
/// );
/// ```
#[cfg(any(feature = "std", feature = "alloc"))]
#[macro_export]
macro_rules! btkramaframe {
    // Multiple entries
    (<$TRES:ty, $PRES:ty>
        $(
            $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ;
        )+
    ) => {{
        let mut krama = $crate::KramaFrame::<
            $crate::BTreeMap<&'static str, $crate::keyframe::KeyFrameFunction>,
            $crate::BTreeMap<&'static str, $crate::keylist::KeyList<$TRES, $PRES>>
        >::default();

        $(
             krama.classlist.insert($class, $crate::keyframe::KeyFrameFunction::$easing);
             $(
                krama.insert_new_id(
                    $class,
                    $key as u32,
                    <$TRES as $crate::keylist::TimingResolution>::from_sec($duration as f32)
                );
             )*
        )+

        krama
    }};

    // Single entry (convenience for no trailing semicolon)
    (<$TRES:ty, $PRES:ty> $class:literal $easing:ident [ $($key:literal),* $(,)? ] $duration:literal s ) => {{
        let mut krama = $crate::KramaFrame::<
            $crate::BTreeMap<&'static str, $crate::keyframe::KeyFrameFunction>,
            $crate::BTreeMap<&'static str, $crate::keylist::KeyList<$TRES, $PRES>>
        >::default();

        krama.classlist.insert($class, $crate::keyframe::KeyFrameFunction::$easing);
        $(
            krama.insert_new_id(
                $class,
                $key as u32,
                <$TRES as $crate::keylist::TimingResolution>::from_sec($duration as f32)
            );
        )*
        krama
    }};
}
