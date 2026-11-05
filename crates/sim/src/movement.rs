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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMode {
    Ground,
    Sprint,
    Air,
    Glide,
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
    matches!(mode, MoveMode::Ground | MoveMode::Sprint)
}

fn ground_step(params: &MovementParams, state: &mut MoverState, input: &MoveInput) {
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
    }
    move_horizontal(state, input, params.air_speed);
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
}
