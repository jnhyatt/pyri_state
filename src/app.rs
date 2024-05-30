use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{
    schedule::{InternedSystemSet, SystemSet},
    world::FromWorld,
};

use crate::{
    schedule::{
        schedule_apply_flush, schedule_bevy_state, schedule_detect_change, schedule_resolve_state,
        schedule_send_event, StateFlush, StateFlushEvent, StateFlushSet,
    },
    state::{BevyState, CurrentState, GetState, RawState, SetState},
    storage::StateStorage,
};

pub struct PyriStatePlugin;

impl Plugin for PyriStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(StateFlush)
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);
    }
}

pub trait AppExtPyriState {
    fn add_state_<S: AddState>(&mut self) -> &mut Self;

    fn init_state_<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld;

    fn insert_state_<T: AddStateStorage>(&mut self, storage: T) -> &mut Self;
}

impl AppExtPyriState for App {
    fn add_state_<S: AddState>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::AddStorage::add_state_storage(self, None);
            S::add_state(self);
        }
        self
    }

    fn init_state_<S: AddState>(&mut self) -> &mut Self
    where
        S::AddStorage: FromWorld,
    {
        if !self.world.contains_resource::<CurrentState<S>>() {
            let storage = S::AddStorage::from_world(&mut self.world);
            S::AddStorage::add_state_storage(self, Some(storage));
            S::add_state(self);
        }
        self
    }

    fn insert_state_<T: AddStateStorage>(&mut self, storage: T) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<T::AddState>>() {
            T::add_state_storage(self, Some(storage));
            T::AddState::add_state(self);
        }
        self
    }
}

pub trait AddStateStorage: StateStorage + Sized {
    type AddState: AddState;

    fn add_state_storage(app: &mut App, storage: Option<Self>);
}

pub trait AddState: RawState {
    type AddStorage: AddStateStorage;

    fn add_state(app: &mut App);
}

pub struct ResolveStatePlugin<S: RawState> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: RawState> Plugin for ResolveStatePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_resolve_state::<S>(
            app.get_schedule_mut(StateFlush).unwrap(),
            &self.after,
            &self.before,
        );
    }
}

impl<S: RawState> Default for ResolveStatePlugin<S> {
    fn default() -> Self {
        Self {
            after: Vec::new(),
            before: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<S: RawState> ResolveStatePlugin<S> {
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }

    pub fn after<T: RawState>(mut self) -> Self {
        self.after.push(StateFlushSet::<T>::Resolve.intern());
        self
    }

    pub fn before<T: RawState>(mut self) -> Self {
        self.before.push(StateFlushSet::<T>::Resolve.intern());
        self
    }
}

pub struct DetectChangePlugin<S: GetState + Eq>(PhantomData<S>);

impl<S: GetState + Eq> Plugin for DetectChangePlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: GetState + Eq> Default for DetectChangePlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct FlushEventPlugin<S: GetState + Clone>(PhantomData<S>);

impl<S: GetState + Clone> Plugin for FlushEventPlugin<S> {
    fn build(&self, app: &mut App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_send_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: GetState + Clone> Default for FlushEventPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct BevyStatePlugin<S: GetState + SetState + Clone + PartialEq + Eq + Hash + Debug>(
    PhantomData<S>,
);

impl<S: GetState + SetState + Clone + PartialEq + Eq + Hash + Debug> Plugin for BevyStatePlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_state::<BevyState<S>>();
        schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: GetState + SetState + Clone + PartialEq + Eq + Hash + Debug> Default
    for BevyStatePlugin<S>
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct ApplyFlushPlugin<S: GetState + Clone>(PhantomData<S>);

impl<S: GetState + Clone> Plugin for ApplyFlushPlugin<S> {
    fn build(&self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: GetState + Clone> Default for ApplyFlushPlugin<S> {
    fn default() -> Self {
        Self(PhantomData)
    }
}
