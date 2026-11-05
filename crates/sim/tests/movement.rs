//! Scripted movement scenario: sprint, slide, jump, glide, land.

use sim::movement::{MoveInput, MoveMode, MovementParams, MoverState, step};

fn params() -> MovementParams {
    MovementParams {
        walk_speed: 120,
        sprint_speed: 420,
        air_speed: 360,
        jump_speed: 260,
        gravity: 14,
        glide_fall_speed: 40,
        max_stamina: 120,
        sprint_stamina_cost: 4,
        stamina_regen: 6,
        slide_speed: 560,
        slide_ticks: 20,
        slide_stamina_cost: 30,
        wall_run_speed: 400,
        wall_run_ticks: 40,
    }
}

fn script() -> Vec<MoveInput> {
    let forward = MoveInput {
        dir_x: 1,
        ..MoveInput::default()
    };
    let sprint = MoveInput {
        sprint: true,
        ..forward
    };
    let slide = MoveInput {
        slide: true,
        ..forward
    };
    let jump = MoveInput {
        jump: true,
        ..sprint
    };
    let glide = MoveInput {
        glide: true,
        ..forward
    };
    let mut inputs = vec![sprint; 10];
    inputs.push(slide);
    inputs.extend(vec![forward; 25]);
    inputs.extend(vec![sprint; 5]);
    inputs.push(jump);
    inputs.extend(vec![glide; 120]);
    inputs
}

fn run(params: &MovementParams) -> Vec<MoverState> {
    let mut state = MoverState::new(params);
    script()
        .iter()
        .map(|input| {
            state = step(params, &state, input);
            state
        })
        .collect()
}

#[test]
fn scenario_is_deterministic() {
    assert_eq!(run(&params()), run(&params()));
}

#[test]
fn scenario_never_goes_underground_or_overdraws_stamina() {
    for state in run(&params()) {
        assert!(state.z >= 0);
        assert!((0..=params().max_stamina).contains(&state.stamina));
    }
}

#[test]
fn scenario_covers_every_ground_and_air_mode() {
    let modes: Vec<MoveMode> = run(&params()).iter().map(|s| s.mode).collect();
    assert!(modes.contains(&MoveMode::Sprint));
    assert!(modes.iter().any(|m| matches!(m, MoveMode::Slide { .. })));
    assert!(modes.contains(&MoveMode::Air));
    assert!(modes.contains(&MoveMode::Glide));
}

#[test]
fn scenario_ends_on_the_ground_far_from_the_start() {
    let states = run(&params());
    let last = states[states.len() - 1];
    assert_eq!((last.z, last.mode), (0, MoveMode::Ground));
    assert!(last.x > 20_000);
}
