use std::{convert::Infallible, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy_ecs::{
    event::Event,
    schedule::{
        InternedSystemSet, IntoSystemConfigs, IntoSystemSetConfigs, NextState, Schedule,
        ScheduleLabel, SystemSet,
    },
    system::{Res, ResMut},
};

use crate::{
    buffer::NextState_,
    state::{StateExtClone, StateExtEq, State_},
    util::BevyState,
};

#[derive(ScheduleLabel, Clone, Hash, PartialEq, Eq, Debug)]
pub struct StateFlush;

// Provides system ordering for state flush handling systems.
#[derive(SystemSet)]
pub enum StateFlushSet<S: State_> {
    Resolve,
    Trigger,
    Flush,
    Exit,
    Transition,
    Enter,
    _PhantomData(PhantomData<S>, Infallible),
}

impl<S: State_> Clone for StateFlushSet<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Resolve => Self::Resolve,
            Self::Trigger => Self::Trigger,
            Self::Flush => Self::Flush,
            Self::Exit => Self::Exit,
            Self::Transition => Self::Transition,
            Self::Enter => Self::Enter,
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

impl<S: State_> PartialEq for StateFlushSet<S> {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

impl<S: State_> Eq for StateFlushSet<S> {}

impl<S: State_> Hash for StateFlushSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S: State_> Debug for StateFlushSet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolve => write!(f, "Resolve"),
            Self::Trigger => write!(f, "Trigger"),
            Self::Flush => write!(f, "Flush"),
            Self::Exit => write!(f, "Exit"),
            Self::Transition => write!(f, "Transition"),
            Self::Enter => write!(f, "Enter"),
            Self::_PhantomData(..) => unreachable!(),
        }
    }
}

#[derive(SystemSet, Clone, Hash, PartialEq, Eq, Debug)]
struct ApplyFlushSet;

#[derive(Event)]
pub struct StateFlushEvent<S: State_> {
    pub before: Option<S>,
    pub after: Option<S>,
}

fn check_flush_flag<S: State_>(state: Res<NextState_<S>>) -> bool {
    state.flush
}

pub fn schedule_detect_change<S: State_ + Eq>(schedule: &mut Schedule) {
    schedule.add_systems(
        S::set_flush(true)
            .run_if(S::will_any_change)
            .in_set(StateFlushSet::<S>::Trigger),
    );
}

pub fn schedule_resolve_state<S: State_>(
    schedule: &mut Schedule,
    after: &[InternedSystemSet],
    before: &[InternedSystemSet],
) {
    // External ordering
    for &system_set in after {
        schedule.configure_sets(StateFlushSet::<S>::Resolve.after(system_set));
    }
    for &system_set in before {
        schedule.configure_sets(StateFlushSet::<S>::Resolve.before(system_set));
    }

    // Internal ordering
    schedule.configure_sets((
        StateFlushSet::<S>::Resolve.before(ApplyFlushSet),
        (
            StateFlushSet::<S>::Trigger,
            StateFlushSet::<S>::Flush.run_if(check_flush_flag::<S>),
        )
            .chain()
            .in_set(StateFlushSet::<S>::Resolve),
        (
            StateFlushSet::<S>::Exit.run_if(S::will_any_exit),
            StateFlushSet::<S>::Transition.run_if(S::will_any_transition),
            StateFlushSet::<S>::Enter.run_if(S::will_any_enter),
        )
            .chain()
            .in_set(StateFlushSet::<S>::Flush),
    ));
}

pub fn schedule_send_event<S: State_ + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(S::on_any_flush(S::send_flush_event));
}

pub fn schedule_apply_flush<S: State_ + Clone>(schedule: &mut Schedule) {
    schedule.add_systems(
        (S::apply_flush, S::set_flush(false))
            .run_if(check_flush_flag::<S>)
            .in_set(ApplyFlushSet),
    );
}

pub fn schedule_bevy_state<S: State_ + Clone + PartialEq + Eq + Hash + Debug>(
    schedule: &mut Schedule,
) {
    let update_bevy_state =
        |pyri_state: Res<NextState_<S>>, mut bevy_state: ResMut<NextState<BevyState<S>>>| {
            if bevy_state.0.is_none() {
                bevy_state.set(BevyState(pyri_state.get().cloned()));
            }
        };

    let update_pyri_state = |mut pyri_state: ResMut<NextState_<S>>,
                             bevy_state: Res<NextState<BevyState<S>>>| {
        if let Some(value) = bevy_state.0.clone() {
            pyri_state.set_flush(true).inner = value.0;
        }
    };

    schedule.add_systems((
        update_pyri_state.in_set(StateFlushSet::<S>::Trigger),
        S::on_any_flush(update_bevy_state),
    ));
}
