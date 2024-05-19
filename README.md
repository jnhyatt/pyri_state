`pyri_state` is an experimental 3rd-party alternative to `bevy_state`. In `pyri_state`, states are simple double-buffered resources with a fixed flush point and some tooling around change detection and system ordering.

# Features

1. **Computed & sub states:** Roll your own computed and sub states with the full power of bevy systems.
2. **Conditional scheduling:** Configure systems to handle arbitrary state transitions via run conditions.
3. **Partial mutation:** Mutate the next state directly instead of replacing it with an entirely new value.
4. **Refresh transition:** Trigger a transition from the current state to itself.
5. **Remove & insert triggers:** Trigger a state to be removed or inserted next frame.
6. **Per-state settings:** Configure the state transition behavior per state type.
7. **Ergonomic improvements:** See below.

# Example code

Using `bevy_state`:

```rust
#[derive(States, Clone, PartialEq, Eq, Hash, Debug)]
enum GameState { ... }

fn frobnicate(
    game_state: Res<State<GameState>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) { ... }

enum ShouldSpawnEasterEggState(bool);

impl ComputedState for ShouldSpawnEasterEggState {
    type SourceStates = GameState;
    
    // Can't access the previous state or use any system params in here, and there's no way
    // to trigger or handle same-state transitions (aka refresh transitions) on GameState.
    fn compute(sources: GameState) -> Option<Self> { ... }
}

app.add_systems(OnEnter(ShouldSpawnEasterEggState(true)), spawn_easter_egg);
```

Using `pyri_state`:

```rust
#[derive(State, Clone, PartialEq, Eq)]
enum GameState { ... }

fn frobnicate(mut game_state: StateMut<GameState>) { ... }

fn should_spawn_easter_egg(game_state: StateRef<GameState>) -> bool {
    let (current, next) = game_state.unwrap();
    ...
}

app.add_systems(
    StateTransition,
    spawn_easter_egg.run_if(should_spawn_easter_egg).in_set(OnTrans::<GameState>::Apply),
);
```

# Remaining tasks

- [ ] Implement per-state settings via the `State` trait and derive macro.
- [ ] How does flushing states once per frame interact with `FixedUpdate`?
- [ ] Include a test or example for each mentioned feature.
- [ ] Write documentation.
