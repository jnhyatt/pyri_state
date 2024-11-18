//! Configure [`ResolveStateSet`] system sets.

#[cfg(feature = "bevy_app")]
pub use app::*;

#[cfg(feature = "bevy_app")]
mod app {
    use std::marker::PhantomData;

    use bevy_app::{App, Plugin};
    use bevy_ecs::schedule::{InternedSystemSet, SystemSet};

    use crate::state::State;

    use super::{schedule_resolve_state, ResolveStateSet};

    /// A plugin that configures the [`ResolveStateSet<S>`] system sets for the [`State`]
    /// type `S` in the [`StateFlush`](crate::schedule::StateFlush) schedule.
    ///
    /// To specify a dependency relative to another `State` type `T`, add
    /// [`ResolveStateSet::<T>::Resolve`] to [`after`](Self::after) or [`before`](Self::before).
    ///
    /// Calls [`schedule_resolve_state<S>`].
    pub struct ResolveStatePlugin<S: State> {
        after: Vec<InternedSystemSet>,
        before: Vec<InternedSystemSet>,
        _phantom: PhantomData<S>,
    }

    impl<S: State> Plugin for ResolveStatePlugin<S> {
        fn build(&self, app: &mut App) {
            schedule_resolve_state::<S>(
                app.get_schedule_mut(crate::schedule::StateFlush).unwrap(),
                &self.after,
                &self.before,
            );
        }
    }

    impl<S: State> Default for ResolveStatePlugin<S> {
        fn default() -> Self {
            Self {
                after: Vec::new(),
                before: Vec::new(),
                _phantom: PhantomData,
            }
        }
    }

    impl<S: State> ResolveStatePlugin<S> {
        /// Create a [`ResolveStatePlugin`] from `.after` and `.before` system sets.
        pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
            Self {
                after,
                before,
                _phantom: PhantomData,
            }
        }

        /// Configure a `.after` system set.
        pub fn after<T: State>(mut self) -> Self {
            self.after.push(ResolveStateSet::<T>::Resolve.intern());
            self
        }

        /// Configure a `.before` system set.
        pub fn before<T: State>(mut self) -> Self {
            self.before.push(ResolveStateSet::<T>::Resolve.intern());
            self
        }
    }
}

use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::schedule::{Condition, InternedSystemSet, IntoSystemSetConfigs, Schedule, SystemSet};

use crate::{schedule::ApplyFlushSet, state::State};

/// A suite of system sets in the [`StateFlush`](crate::schedule::StateFlush)
/// schedule for each [`State`] type `S`.
///
/// Configured [by default](pyri_state_derive::State) by
/// [`ResolveStatePlugin<S>`] as follows:
///
/// 1. [`Resolve`](Self::Resolve) (before or after other `Resolve` system sets based on
///    state dependencies, and before [`ApplyFlushSet`])
///     1. [`Compute`](Self::Compute)
///     2. [`Trigger`](Self::Trigger)
///     3. [`Flush`](Self::Flush) (and [`AnyFlush`](Self::AnyFlush) if the global state will flush)
///         1. [`Exit`](Self::Exit) (and [`AnyExit`](Self::AnyExit) if the global state will exit)
///         2. [`Trans`](Self::Trans) (and [`AnyTrans`](Self::AnyTrans) if the global state will
///            transition)
///         3. [`Enter`](Self::Enter) (and [`AnyEnter`](Self::AnyEnter) if the global state will
///            enter)
#[derive(SystemSet)]
pub enum ResolveStateSet<S: State> {
    /// Resolve the state flush logic for `S`.
    Resolve,
    /// Optionally compute the next value for `S`.
    Compute,
    /// Decide whether to trigger a flush for `S`.
    Trigger,
    /// Run on-flush hooks for `S`.
    Flush,
    /// Run on-exit hooks for `S`.
    Exit,
    /// Run on-transition hooks for `S`.
    Trans,
    /// Run on-enter hooks for `S`.
    Enter,
    /// Run global on-flush hooks for `S`.
    AnyFlush,
    /// Run global on-exit hooks for `S`.
    AnyExit,
    /// Run global on-transition hooks for `S`.
    AnyTrans,
    /// Run global on-enter hooks for `S.
    AnyEnter,
    #[doc(hidden)]
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State> Clone for ResolveStateSet<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolve => Self::Resolve,
            Self::Compute => Self::Compute,
            Self::Trigger => Self::Trigger,
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Trans => Self::Trans,
            Self::Enter => Self::Enter,
            Self::AnyFlush => Self::AnyFlush,
            Self::AnyExit => Self::AnyExit,
            Self::AnyTrans => Self::AnyTrans,
            Self::AnyEnter => Self::AnyEnter,
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

impl<S: State> PartialEq for ResolveStateSet<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State> Eq for ResolveStateSet<S> {}

impl<S: State> Hash for ResolveStateSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State> Debug for ResolveStateSet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve => write!(f, "Resolve"),
            Self::Compute => write!(f, "Compute"),
            Self::Trigger => write!(f, "Trigger"),
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Trans => write!(f, "Trans"),
            Self::Enter => write!(f, "Enter"),
            Self::AnyFlush => write!(f, "AnyFlush"),
            Self::AnyExit => write!(f, "AnyExit"),
            Self::AnyTrans => write!(f, "AnyTrans"),
            Self::AnyEnter => write!(f, "AnyEnter"),
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

/// Configure [`ResolveStateSet<S>`] for the [`State`] type `S` in a schedule.
///
/// To specify a dependency relative to another `State` type `T`, include
/// [`ResolveStateSet::<T>::Resolve`] in `after` or `before`.
///
/// Used in [`ResolveStatePlugin<S>`].
pub fn schedule_resolve_state<S: State>(
    schedule: &mut Schedule,
    after: &[InternedSystemSet],
    before: &[InternedSystemSet],
) {
    // External ordering
    for &system_set in after {
        schedule.configure_sets(ResolveStateSet::<S>::Resolve.after(system_set));
    }
    for &system_set in before {
        schedule.configure_sets(ResolveStateSet::<S>::Resolve.before(system_set));
    }

    // Internal ordering
    schedule.configure_sets((
        ResolveStateSet::<S>::Resolve.before(ApplyFlushSet),
        (
            ResolveStateSet::<S>::Compute,
            // Logic in this system set should only run if not triggered.
            ResolveStateSet::<S>::Trigger,
            // Logic in this system set should only run if triggered.
            ResolveStateSet::<S>::Flush,
        )
            .chain()
            .in_set(ResolveStateSet::<S>::Resolve),
        (
            ResolveStateSet::<S>::Exit,
            ResolveStateSet::<S>::Trans,
            ResolveStateSet::<S>::Enter,
        )
            .chain()
            .in_set(ResolveStateSet::<S>::Flush),
        ResolveStateSet::<S>::AnyFlush
            .run_if(S::is_triggered)
            .in_set(ResolveStateSet::<S>::Flush),
        (
            ResolveStateSet::<S>::AnyExit
                .run_if(S::is_enabled)
                .in_set(ResolveStateSet::<S>::Exit),
            ResolveStateSet::<S>::AnyTrans
                .run_if(S::is_enabled.and(S::will_be_enabled))
                .in_set(ResolveStateSet::<S>::Trans),
            ResolveStateSet::<S>::AnyEnter
                .run_if(S::will_be_enabled)
                .in_set(ResolveStateSet::<S>::Enter),
        )
            .in_set(ResolveStateSet::<S>::AnyFlush),
    ));
}
