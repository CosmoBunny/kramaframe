use std::{
    collections::BTreeMap,
    ops::{Add, Mul, Range, RangeInclusive, Sub},
};

use crate::{
    keyframe::KeyFrameFunction,
    keylist::{GetValueByGeneric, GetValueByRange, KeyList, ProgressResolution, TimingResolution},
};

pub mod prelude {
    pub use crate::keyframe::KeyFrameFunction;
    pub use crate::keylist::{
        GetValueByGeneric, GetValueByRange, KeyList, ProgressResolution, TimingResolution,
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

impl<TRES: TimingResolution, PRES: ProgressResolution + Eq> Default
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

impl<TRES: TimingResolution, PRES: ProgressResolution + Eq>
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
    pub fn get_progress_f32(&mut self, classname: &'static str, id: u32) -> f32 {
        if let Some(keylist) = self.framelist.get_mut(classname) {
            if let Some(progresslist) = keylist.get_mut(id) {
                progresslist.get_progress_f32()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Gets the current elapsed time of a specific animation instance in seconds.
    ///
    /// Returns `0.0` if the class name or id is not found.
    pub fn get_time_f32(&mut self, classname: &'static str, id: u32) -> f32 {
        if let Some(keylist) = self.framelist.get_mut(classname) {
            if let Some(progresslist) = keylist.get_mut(id) {
                progresslist.get_time_f32()
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Calculates and returns an interpolated value for an animation within a given `Range`.
    ///
    /// The interpolation is based on the animation's current progress and its class's `KeyFrameFunction`.
    /// This method is suitable for types that support the necessary arithmetic operations (`Add`, `Sub`, `Mul<f32>`).
    ///
    /// Returns the `range.start` value if the class name or id is not found.
    pub fn get_value_byrange<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        range: Range<T>,
    ) -> T
    where
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    progresslist.get_value_byrange(range, keyframe)
                } else {
                    range.start
                }
            } else {
                range.start
            }
        } else {
            range.start
        }
    }

    /// Calculates and returns an interpolated value for an animation within a given `RangeInclusive`.
    ///
    /// The interpolation is based on the animation's current progress and its class's `KeyFrameFunction`.
    /// This method is suitable for types that support the necessary arithmetic operations (`Add`, `Sub`, `Mul<f32>`).
    ///
    /// Returns the `range.start()` value if the class name or id is not found.
    pub fn get_value_byrange_inclusive<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Sized + Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    progresslist.get_value_byrangeinclusive(range, keyframe)
                } else {
                    range.start().clone()
                }
            } else {
                range.start().clone()
            }
        } else {
            range.start().clone()
        }
    }

    /// Calculates and returns an interpolated value within a `RangeInclusive` for types that implement `Copy`.
    ///
    /// This is a more generic version of `get_value_byrange_inclusive`, but relies on the
    /// underlying `ProgressList`'s implementation of `GetValueByRange<T>`.
    ///
    /// Returns the `range.start()` value if the class name or id is not found.
    pub fn get_generic_value_by_range<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByRange<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    progresslist.get_value_byrangeinclusive(range, keyframe)
                } else {
                    range.start().clone()
                }
            } else {
                range.start().clone()
            }
        } else {
            range.start().clone()
        }
    }

    /// Gets a value from a `Range` based on animation progress, for generic types.
    ///
    /// This method is intended for types that might not support arithmetic interpolation (e.g., enums).
    /// It relies on the `GetValueByGeneric` trait to determine the value based on progress.
    ///
    /// Returns `range.start` if the class name or id is not found.
    pub fn get_generic_byrange<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        range: Range<T>,
    ) -> T
    where
        T: Copy,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    progresslist.get_generic_byrange(range, keyframe)
                } else {
                    range.start
                }
            } else {
                range.start
            }
        } else {
            range.start
        }
    }

    /// Gets a value from a `RangeInclusive` based on animation progress, for generic types.
    ///
    /// This method is intended for types that might not support arithmetic interpolation (e.g., enums).
    /// It relies on the `GetValueByGeneric` trait to determine the value based on progress.
    ///
    /// Returns `range.start()` if the class name or id is not found.
    pub fn get_generic_value_by_rangeinclusive<T>(
        &mut self,
        on_classname: &'static str,
        id: u32,
        range: RangeInclusive<T>,
    ) -> T
    where
        T: Clone,
        crate::keylist::ProgressList<TRES, PRES>: GetValueByGeneric<T>,
    {
        if let Some(keyframe) = self.classlist.get(on_classname) {
            if let Some(keylist) = self.framelist.get_mut(on_classname) {
                if let Some(progresslist) = keylist.get_mut(id) {
                    progresslist.get_generic_byrangeinclusive(range, keyframe)
                } else {
                    range.start().clone()
                }
            } else {
                range.start().clone()
            }
        } else {
            range.start().clone()
        }
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
                    previous
                } else {
                    range.start
                }
            } else {
                range.start
            }
        } else {
            range.start
        }
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
                    previous
                } else {
                    range.start
                }
            } else {
                range.start
            }
        } else {
            range.start
        }
    }
}

/// A type alias for a class list implemented with `BTreeMap`.
pub type BTclasslist = BTreeMap<&'static str, KeyFrameFunction>;
/// A type alias for a frame list implemented with `BTreeMap`.
pub type BTframelist<TRES, PRES> = BTreeMap<&'static str, KeyList<TRES, PRES>>;
