//! Character controller. Fixed-point millimeters, one pure step per tick.
//! Every number comes from `MovementParams`, which is content data.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovementParams {
    /// Speeds are in mm per tick.
    pub walk_speed: i64,
    pub sprint_speed: i64,
    pub air_speed: i64,
    pub jump_speed: i64,
    /// Downward acceleration in mm per tick squared.
    pub gravity: i64,
    pub glide_fall_speed: i64,
    pub max_stamina: i64,
    pub sprint_stamina_cost: i64,
    pub stamina_regen: i64,
    pub slide_speed: i64,
    pub slide_ticks: u32,
    pub slide_stamina_cost: i64,
    pub wall_run_speed: i64,
    pub wall_run_ticks: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMode {
    Ground,
    Sprint,
    Air,
    Glide,
    Slide { ticks_left: u32 },
    WallRun { ticks_left: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoverState {
    pub x: i64,
    pub y: i64,
    /// Height above the ground plane. The ground is at zero.
    pub z: i64,
    pub vz: i64,
    pub mode: MoveMode,
    pub stamina: i64,
    /// Set once a wall-run ended; cleared on landing. Stops endless wall-runs.
    pub wall_run_spent: bool,
}

impl MoverState {
    pub fn new(params: &MovementParams) -> MoverState {
        MoverState {
            x: 0,
            y: 0,
            z: 0,
            vz: 0,
            mode: MoveMode::Ground,
            stamina: params.max_stamina,
            wall_run_spent: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MoveInput {
    /// Direction on each axis. Only the sign is used.
    pub dir_x: i8,
    pub dir_y: i8,
    pub sprint: bool,
    pub jump: bool,
    pub glide: bool,
    pub slide: bool,
    pub wall_contact: bool,
}

impl MoveInput {
    fn is_moving(&self) -> bool {
        self.dir_x != 0 || self.dir_y != 0
    }
}

pub fn step(params: &MovementParams, state: &MoverState, input: &MoveInput) -> MoverState {
    let mut next = *state;
    if is_grounded(next.mode) {
        if !input.jump {
            ground_step(params, &mut next, input);
            return next;
        }
        next.vz = params.jump_speed;
        next.mode = MoveMode::Air;
    }
    air_step(params, &mut next, input);
    next
}

fn is_grounded(mode: MoveMode) -> bool {
    matches!(
        mode,
        MoveMode::Ground | MoveMode::Sprint | MoveMode::Slide { .. }
    )
}

fn ground_step(params: &MovementParams, state: &mut MoverState, input: &MoveInput) {
    if let MoveMode::Slide { ticks_left } = state.mode
        && ticks_left > 1
    {
        state.mode = MoveMode::Slide {
            ticks_left: ticks_left - 1,
        };
        move_horizontal(state, input, params.slide_speed);
        return;
    }
    let can_slide = state.mode == MoveMode::Sprint
        && input.slide
        && input.is_moving()
        && params.slide_ticks > 0
        && state.stamina >= params.slide_stamina_cost;
    if can_slide {
        state.mode = MoveMode::Slide {
            ticks_left: params.slide_ticks,
        };
        state.stamina -= params.slide_stamina_cost;
        move_horizontal(state, input, params.slide_speed);
        return;
    }
    let sprinting = input.sprint && input.is_moving() && state.stamina > 0;
    let speed = if sprinting {
        state.mode = MoveMode::Sprint;
        state.stamina = state
            .stamina
            .saturating_sub(params.sprint_stamina_cost)
            .max(0);
        params.sprint_speed
    } else {
        state.mode = MoveMode::Ground;
        state.stamina = state
            .stamina
            .saturating_add(params.stamina_regen)
            .min(params.max_stamina);
        params.walk_speed
    };
    move_horizontal(state, input, speed);
}

fn air_step(params: &MovementParams, state: &mut MoverState, input: &MoveInput) {
    if wall_run_step(params, state, input) {
        return;
    }
    state.vz = state.vz.saturating_sub(params.gravity);
    state.mode = MoveMode::Air;
    if input.glide && state.vz < 0 {
        state.vz = state.vz.max(-params.glide_fall_speed);
        state.mode = MoveMode::Glide;
    }
    state.z = state.z.saturating_add(state.vz);
    if state.z <= 0 {
        state.z = 0;
        state.vz = 0;
        state.mode = MoveMode::Ground;
        state.wall_run_spent = false;
    }
    move_horizontal(state, input, params.air_speed);
}

/// Returns true when the wall-run handled this tick.
fn wall_run_step(params: &MovementParams, state: &mut MoverState, input: &MoveInput) -> bool {
    let left = match state.mode {
        MoveMode::WallRun { ticks_left } => ticks_left,
        _ if !state.wall_run_spent => params.wall_run_ticks,
        _ => 0,
    };
    if input.wall_contact && input.sprint && left > 0 {
        state.mode = MoveMode::WallRun {
            ticks_left: left - 1,
        };
        state.vz = 0;
        move_horizontal(state, input, params.wall_run_speed);
        return true;
    }
    if matches!(state.mode, MoveMode::WallRun { .. }) {
        state.wall_run_spent = true;
    }
    false
}

fn move_horizontal(state: &mut MoverState, input: &MoveInput, speed: i64) {
    let speed = if input.dir_x != 0 && input.dir_y != 0 {
        speed * 707 / 1000
    } else {
        speed
    };
    state.x = state
        .x
        .saturating_add(i64::from(input.dir_x.signum()) * speed);
    state.y = state
        .y
        .saturating_add(i64::from(input.dir_y.signum()) * speed);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> MovementParams {
        MovementParams {
            walk_speed: 100,
            sprint_speed: 300,
            air_speed: 250,
            jump_speed: 200,
            gravity: 20,
            glide_fall_speed: 30,
            max_stamina: 100,
            sprint_stamina_cost: 10,
            stamina_regen: 5,
            slide_speed: 400,
            slide_ticks: 3,
            slide_stamina_cost: 20,
            wall_run_speed: 280,
            wall_run_ticks: 4,
        }
    }

    fn forward() -> MoveInput {
        MoveInput {
            dir_x: 1,
            ..MoveInput::default()
        }
    }

    fn run(state: MoverState, input: MoveInput, ticks: usize) -> MoverState {
        (0..ticks).fold(state, |s, _| step(&params(), &s, &input))
    }

    #[test]
    fn walking_moves_at_walk_speed() {
        let state = step(&params(), &MoverState::new(&params()), &forward());
        assert_eq!((state.x, state.mode), (100, MoveMode::Ground));
    }

    #[test]
    fn sprinting_is_faster_and_drains_stamina() {
        let input = MoveInput {
            sprint: true,
            ..forward()
        };
        let state = step(&params(), &MoverState::new(&params()), &input);
        assert_eq!(
            (state.x, state.stamina, state.mode),
            (300, 90, MoveMode::Sprint)
        );
    }

    #[test]
    fn empty_stamina_falls_back_to_walking() {
        let input = MoveInput {
            sprint: true,
            ..forward()
        };
        let state = run(MoverState::new(&params()), input, 10);
        assert_eq!(state.stamina, 0);
        let next = step(&params(), &state, &input);
        assert_eq!(next.x - state.x, 100);
        assert_eq!(next.stamina, 5);
    }

    #[test]
    fn stamina_regenerates_up_to_the_cap() {
        let mut state = MoverState::new(&params());
        state.stamina = 98;
        assert_eq!(run(state, MoveInput::default(), 5).stamina, 100);
    }

    #[test]
    fn diagonal_movement_is_not_faster() {
        let input = MoveInput {
            dir_x: 1,
            dir_y: -1,
            ..MoveInput::default()
        };
        let state = step(&params(), &MoverState::new(&params()), &input);
        assert_eq!((state.x, state.y), (70, -70));
    }

    #[test]
    fn jump_leaves_the_ground_and_lands_again() {
        let jump = MoveInput {
            jump: true,
            ..MoveInput::default()
        };
        let airborne = step(&params(), &MoverState::new(&params()), &jump);
        assert_eq!(airborne.mode, MoveMode::Air);
        assert_eq!(airborne.z, 180);
        let landed = run(airborne, MoveInput::default(), 40);
        assert_eq!((landed.z, landed.vz, landed.mode), (0, 0, MoveMode::Ground));
    }

    #[test]
    fn air_control_uses_air_speed() {
        let input = MoveInput {
            jump: true,
            ..forward()
        };
        let state = step(&params(), &MoverState::new(&params()), &input);
        assert_eq!(state.x, 250);
    }

    #[test]
    fn gliding_caps_the_fall_speed() {
        let mut high = MoverState::new(&params());
        high.z = 10_000;
        high.mode = MoveMode::Air;
        let glide = MoveInput {
            glide: true,
            ..MoveInput::default()
        };
        let gliding = run(high, glide, 20);
        let falling = run(high, MoveInput::default(), 20);
        assert_eq!(gliding.mode, MoveMode::Glide);
        assert_eq!(gliding.vz, -30);
        assert!(gliding.z > falling.z);
    }

    #[test]
    fn glide_does_nothing_while_rising() {
        let input = MoveInput {
            jump: true,
            glide: true,
            ..MoveInput::default()
        };
        let state = step(&params(), &MoverState::new(&params()), &input);
        assert_eq!(state.mode, MoveMode::Air);
    }

    fn sprinting_state() -> MoverState {
        step(
            &params(),
            &MoverState::new(&params()),
            &MoveInput {
                sprint: true,
                ..forward()
            },
        )
    }

    #[test]
    fn sprint_slide_runs_for_its_duration() {
        let slide = MoveInput {
            slide: true,
            ..forward()
        };
        let mut state = step(&params(), &sprinting_state(), &slide);
        assert_eq!(state.mode, MoveMode::Slide { ticks_left: 3 });
        assert_eq!(state.stamina, 70);
        let start_x = state.x;
        state = step(&params(), &state, &forward());
        state = step(&params(), &state, &forward());
        assert_eq!(state.mode, MoveMode::Slide { ticks_left: 1 });
        assert_eq!(state.x - start_x, 800);
        state = step(&params(), &state, &forward());
        assert_eq!(state.mode, MoveMode::Ground);
    }

    #[test]
    fn slide_needs_a_sprint_and_stamina() {
        let slide = MoveInput {
            slide: true,
            ..forward()
        };
        let walking = step(&params(), &MoverState::new(&params()), &slide);
        assert_eq!(walking.mode, MoveMode::Ground);
        let mut tired = sprinting_state();
        tired.stamina = 19;
        assert_ne!(
            step(&params(), &tired, &slide).mode,
            MoveMode::Slide { ticks_left: 3 }
        );
    }

    #[test]
    fn jumping_cancels_a_slide() {
        let slide = MoveInput {
            slide: true,
            ..forward()
        };
        let sliding = step(&params(), &sprinting_state(), &slide);
        let jump = MoveInput {
            jump: true,
            ..forward()
        };
        assert_eq!(step(&params(), &sliding, &jump).mode, MoveMode::Air);
    }

    fn wall_run_input() -> MoveInput {
        MoveInput {
            sprint: true,
            wall_contact: true,
            ..forward()
        }
    }

    fn airborne(height: i64) -> MoverState {
        let mut state = MoverState::new(&params());
        state.z = height;
        state.mode = MoveMode::Air;
        state
    }

    #[test]
    fn wall_run_holds_height_then_drops() {
        let mut state = airborne(1_000);
        for _ in 0..4 {
            state = step(&params(), &state, &wall_run_input());
            assert_eq!(state.z, 1_000);
        }
        assert_eq!(state.x, 4 * 280);
        state = step(&params(), &state, &wall_run_input());
        assert!(state.z < 1_000);
        assert_eq!(state.mode, MoveMode::Air);
    }

    #[test]
    fn wall_run_cannot_restart_before_landing() {
        let mut state = airborne(10_000);
        for _ in 0..6 {
            state = step(&params(), &state, &wall_run_input());
        }
        assert!(state.wall_run_spent);
        let height = state.z;
        state = step(&params(), &state, &wall_run_input());
        assert!(state.z < height);
    }

    #[test]
    fn landing_clears_the_wall_run_lock() {
        let mut state = airborne(100);
        state.wall_run_spent = true;
        let landed = run(state, MoveInput::default(), 20);
        assert_eq!(landed.mode, MoveMode::Ground);
        assert!(!landed.wall_run_spent);
    }
}
