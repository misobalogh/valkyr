//! Command line entry point. For now it only hosts two interactive demos of
//! the sim crate: `move` and `wolf`.

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use sim::damage::{DamageType, Vitals};
use sim::movement::{MoveInput, MovementParams, MoverState, step};
use sim::poise::Poise;
use sim::status::{StatusEffect, StatusSet};

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let result = match command.as_deref() {
        Some("move") => run_lines(move_demo),
        Some("wolf") => run_lines(wolf_demo),
        _ => {
            eprintln!("usage: tools <move|wolf>");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run_lines<D: Demo>(make: fn() -> D) -> io::Result<()> {
    let mut demo = make();
    println!("{}", demo.help());
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim() == "q" {
            break;
        }
        println!("{}", demo.line(&line));
        stdout.flush()?;
    }
    Ok(())
}

trait Demo {
    fn help(&self) -> String;
    fn line(&mut self, line: &str) -> String;
}

struct MoveDemo {
    params: MovementParams,
    state: MoverState,
}

fn move_demo() -> MoveDemo {
    let params = MovementParams {
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
    };
    MoveDemo {
        state: MoverState::new(&params),
        params,
    }
}

impl Demo for MoveDemo {
    fn help(&self) -> String {
        "One line = one tick (60 per second). Words: f b l r (direction), sprint, slide, jump, glide, wall.\n\
         Prefix a word with a number to repeat the tick, e.g. `10 f sprint`. Empty line = idle. q = quit."
            .to_string()
    }

    fn line(&mut self, line: &str) -> String {
        let mut words: Vec<&str> = line.split_whitespace().collect();
        let repeat = words
            .first()
            .and_then(|w| w.parse::<usize>().ok())
            .map(|n| {
                words.remove(0);
                n.clamp(1, 600)
            })
            .unwrap_or(1);
        let input = parse_move_input(&words);
        for _ in 0..repeat {
            self.state = step(&self.params, &self.state, &input);
        }
        let s = &self.state;
        format!(
            "x={} y={} z={} mode={:?} stamina={}",
            s.x, s.y, s.z, s.mode, s.stamina
        )
    }
}

fn parse_move_input(words: &[&str]) -> MoveInput {
    let has = |name: &str| words.contains(&name);
    MoveInput {
        dir_x: i8::from(has("f")) - i8::from(has("b")),
        dir_y: i8::from(has("r")) - i8::from(has("l")),
        sprint: has("sprint"),
        jump: has("jump"),
        glide: has("glide"),
        slide: has("slide"),
        wall_contact: has("wall"),
    }
}

struct WolfDemo {
    vitals: Vitals,
    poise: Poise,
    statuses: StatusSet,
}

fn wolf_demo() -> WolfDemo {
    let mut vitals = Vitals::new(200);
    vitals.resistances.insert(DamageType::Frost, 500);
    vitals.resistances.insert(DamageType::Fire, -500);
    WolfDemo {
        vitals,
        poise: Poise::new(100, 2),
        statuses: StatusSet::default(),
    }
}

impl Demo for WolfDemo {
    fn help(&self) -> String {
        "A frost wolf (200 hp, resists frost, weak to fire). Commands: sword, fire, frost, wait. q = quit.\n\
         Every command passes one tick."
            .to_string()
    }

    fn line(&mut self, line: &str) -> String {
        let mut log = Vec::new();
        match line.trim() {
            "sword" => {
                let dealt = self.vitals.take_damage(DamageType::Physical, 30);
                log.push(format!("sword hit for {dealt}"));
                if self.poise.absorb(40, 45) {
                    log.push("the wolf is staggered".to_string());
                }
            }
            "fire" | "frost" => {
                let (damage_type, name) = if line.trim() == "fire" {
                    (DamageType::Fire, "burning")
                } else {
                    (DamageType::Frost, "chilled")
                };
                self.statuses.apply(StatusEffect {
                    damage_type,
                    damage_per_tick: 10,
                    ticks_left: 6,
                });
                log.push(format!("the wolf is {name}"));
            }
            "wait" => {}
            other => return format!("unknown command: {other:?}"),
        }
        let dot = self.statuses.tick(&mut self.vitals);
        if dot > 0 {
            log.push(format!("status damage {dot}"));
        }
        self.poise.tick();
        log.push(format!(
            "hp={} poise={} staggered={}",
            self.vitals.health,
            self.poise.current,
            self.poise.is_staggered()
        ));
        if self.vitals.is_dead() {
            log.push("the wolf is dead".to_string());
        }
        log.join(" | ")
    }
}
