// Use run conditions and pattern matching to schedule your state transition systems.

use bevy::prelude::*;
use pyri_state::{prelude::*, state, will_flush};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PyriStatePlugin))
        .add_state_::<Level>()
        .init_resource::<LevelMeta>()
        .add_systems(
            StateFlush,
            // Schedule the basic level teardown / setup.
            Level::ANY.on_edge(tear_down_old_level, load_new_level),
        )
        .add_systems(
            StateFlush,
            // Upon entering a level, also do the following:
            Level::ANY.on_enter((
                // Level 10 is the final boss fight, so play boss music.
                play_boss_music.run_if(Level(10).will_enter()),
                // Levels 4, 7, and 10 are checkpoints, so save progress.
                save_progress.run_if(state!(Level(4 | 7 | 10)).will_enter()),
                // Early levels (0, 1, 2, and 3) introduce the player to the game.
                spawn_tutorial_popup.run_if(Level::with(|x| x.0 < 4).will_enter()),
                // Spawn an easter egg for very specific level transitions.
                spawn_easter_egg.run_if(will_flush!(
                    (Some(Level(x @ (2 | 5..=8))), Some(&Level(y))) if y == 10 - x,
                )),
                // Randomly generate the next level before loading it, if necessary.
                generate_new_level.before(load_new_level).run_if(
                    |level: NextStateRef<Level>, meta: Res<LevelMeta>| {
                        !meta.generated[level.unwrap().0]
                    },
                ),
            )),
        )
        .run();
}

#[derive(State, Clone, PartialEq, Eq)]
struct Level(usize);

#[derive(Resource, Default)]
struct LevelMeta {
    generated: Vec<bool>,
}

// Dummy systems:
fn tear_down_old_level(_level: Res<CurrentState<Level>>) {}
fn load_new_level(_level: NextStateRef<Level>) {}
fn play_boss_music() {}
fn save_progress() {}
fn spawn_tutorial_popup() {}
fn spawn_easter_egg() {}
fn generate_new_level(_level: NextStateRef<Level>) {}
