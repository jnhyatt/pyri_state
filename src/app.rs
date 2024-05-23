use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use bevy_app::{App, MainScheduleOrder, Plugin, PreUpdate};
use bevy_ecs::{all_tuples, schedule::InternedSystemSet, world::FromWorld};

use crate::{
    buffer::{CurrentState, NextState_},
    schedule::{
        schedule_apply_flush, schedule_bevy_state, schedule_detect_change, schedule_resolve_state,
        schedule_send_event, StateFlush, StateFlushEvent,
    },
    state::{BevyState, RawState},
};

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(StateFlush)
            .world
            .resource_mut::<MainScheduleOrder>()
            .insert_after(PreUpdate, StateFlush);
    }
}

pub trait AppExtState {
    fn add_state_<S: GetStateConfig>(&mut self) -> &mut Self;
    fn init_state_<S: GetStateConfig + FromWorld>(&mut self) -> &mut Self;
    fn insert_state_<S: GetStateConfig>(&mut self, value: S) -> &mut Self;
}

impl AppExtState for App {
    fn add_state_<S: GetStateConfig>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::get_config().configure(self);
            self.init_resource::<CurrentState<S>>()
                .init_resource::<NextState_<S>>();
        }
        self
    }

    fn init_state_<S: GetStateConfig + FromWorld>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            let value = S::from_world(&mut self.world);
            S::get_config().configure(self);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState_::present(value));
        }
        self
    }

    fn insert_state_<S: GetStateConfig>(&mut self, value: S) -> &mut Self {
        if !self.world.contains_resource::<CurrentState<S>>() {
            S::get_config().configure(self);
            self.init_resource::<CurrentState<S>>()
                .insert_resource(NextState_::present(value));
        }
        self
    }
}

pub trait GetStateConfig: RawState {
    fn get_config() -> impl ConfigureState;
}

pub trait ConfigureState {
    fn configure(self, app: &mut App);
}

macro_rules! impl_configure_state {
    ($(($param:ident, $value:ident)), *) => {
        impl<$($param: ConfigureState),*> ConfigureState for ($($param,)*) {
            fn configure(self, app: &mut App) {
                let ($($value,)*) = self;
                $($value.configure(app);)*
                let _ = app;
            }
        }
    };
}

all_tuples!(impl_configure_state, 0, 8, T, t);

pub struct StateConfigResolveState<S: RawState> {
    after: Vec<InternedSystemSet>,
    before: Vec<InternedSystemSet>,
    _phantom: PhantomData<S>,
}

impl<S: RawState> ConfigureState for StateConfigResolveState<S> {
    fn configure(self, app: &mut App) {
        schedule_resolve_state::<S>(
            app.get_schedule_mut(StateFlush).unwrap(),
            &self.after,
            &self.before,
        );
    }
}

impl<S: RawState> StateConfigResolveState<S> {
    pub fn new(after: Vec<InternedSystemSet>, before: Vec<InternedSystemSet>) -> Self {
        Self {
            after,
            before,
            _phantom: PhantomData,
        }
    }
}

pub struct StateConfigDetectChange<S: RawState + Eq>(PhantomData<S>);

impl<S: RawState + Eq> ConfigureState for StateConfigDetectChange<S> {
    fn configure(self, app: &mut App) {
        schedule_detect_change::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Eq> StateConfigDetectChange<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct StateConfigSendEvent<S: RawState + Clone>(PhantomData<S>);

impl<S: RawState + Clone> ConfigureState for StateConfigSendEvent<S> {
    fn configure(self, app: &mut App) {
        app.add_event::<StateFlushEvent<S>>();
        schedule_send_event::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone> StateConfigSendEvent<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct StateConfigBevyState<S: RawState + Clone + PartialEq + Eq + Hash + Debug>(
    PhantomData<S>,
);

impl<S: RawState + Clone + PartialEq + Eq + Hash + Debug> ConfigureState
    for StateConfigBevyState<S>
{
    fn configure(self, app: &mut App) {
        app.init_state::<BevyState<S>>();
        schedule_bevy_state::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone + PartialEq + Eq + Hash + Debug> StateConfigBevyState<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct StateConfigApplyFlush<S: RawState + Clone>(PhantomData<S>);

impl<S: RawState + Clone> ConfigureState for StateConfigApplyFlush<S> {
    fn configure(self, app: &mut App) {
        schedule_apply_flush::<S>(app.get_schedule_mut(StateFlush).unwrap());
    }
}

impl<S: RawState + Clone> StateConfigApplyFlush<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}
