// Allows derive macros in unit tests to refer to this crate as `pyri_state`.
extern crate self as pyri_state;

#[cfg(feature = "bevy_app")]
pub mod app;
pub mod buffer;
pub mod extra;
pub mod schedule;
pub mod state;

pub mod prelude {
    #[doc(hidden)]
    #[cfg(feature = "bevy_app")]
    pub use crate::app::{AppExtState, PyriStatePlugin};

    #[doc(hidden)]
    pub use crate::{
        buffer::{CurrentState, NextState_, StateMut, StateRef},
        schedule::*,
        state::*,
    };

    #[doc(hidden)]
    pub use pyri_state_derive::State;
}

#[cfg(test)]
mod tests {
    use bevy_app::App;
    use bevy_ecs::system::{Res, ResMut};

    use crate::{prelude::*, state};

    fn do_stuff_with<T>(x: T) {
        let _ = x;
    }

    #[derive(State, Clone, PartialEq, Eq, Default)]
    enum GameState {
        #[default]
        Splash,
        Title,
        PlayingGame,
    }

    #[derive(State, Clone, PartialEq, Eq, Default)]
    #[state(after(GameState))]
    struct Paused;

    fn unpause() {}

    fn pause() {}

    #[derive(State, Clone, PartialEq, Eq, Default)]
    #[state(after(GameState))]
    struct Level {
        x: usize,
        y: usize,
    }

    fn exit_level(level: Res<CurrentState<Level>>) {
        let level_state = level.unwrap();
        do_stuff_with::<&Level>(level_state);
    }

    fn enter_level(level_state: Res<NextState_<Level>>) {
        let level_state = level_state.unwrap();
        do_stuff_with::<&Level>(level_state);
    }

    #[derive(State, Clone, PartialEq, Eq)]
    #[state(after(Level))]
    enum SquareColor {
        Black,
        White,
    }

    fn compute_color(level: Res<NextState_<Level>>, mut color: ResMut<NextState_<SquareColor>>) {
        color.inner = level.get().map(|level| {
            if level.x + level.y % 2 == 0 {
                SquareColor::Black
            } else {
                SquareColor::White
            }
        });
    }

    fn exit_color(color_state: Res<CurrentState<SquareColor>>) {
        let color_state = color_state.unwrap();
        do_stuff_with::<&SquareColor>(color_state);
    }

    fn enter_color(color_state: Res<NextState_<SquareColor>>) {
        let color_state = color_state.unwrap();
        do_stuff_with::<&SquareColor>(color_state);
    }

    #[test]
    fn foo() {
        let mut app = App::new();

        app.add_plugins(PyriStatePlugin)
            .init_state_::<GameState>()
            .add_state_::<Paused>()
            .add_state_::<Level>()
            .add_state_::<SquareColor>()
            .add_systems(
                StateFlush,
                (
                    GameState::PlayingGame.on_exit((Paused::disable, Level::disable)),
                    GameState::PlayingGame.on_enter((Paused::enable, Level::enable)),
                    Paused.on_exit(unpause),
                    Paused.on_enter(pause),
                    Level::ANY.on_exit(exit_level),
                    Level::ANY.on_enter((enter_level, compute_color)),
                    SquareColor::ANY.on_exit(exit_color),
                    SquareColor::ANY.on_enter(enter_color),
                    state!(Level { x: x @ 3..=8, y } if *y == x + 2).on_exit(exit_level),
                    Level::with(|s| s.x > s.y).on_exit(exit_level),
                ),
            );
    }
}
