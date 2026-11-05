//! Scripted combat scenario against a frost wolf.

use sim::damage::{DamageType, Vitals};
use sim::hitbox::Circle;
use sim::poise::Poise;
use sim::status::{StatusEffect, StatusSet};

fn frost_wolf() -> (Vitals, Poise, StatusSet) {
    let mut vitals = Vitals::new(200);
    vitals.resistances.insert(DamageType::Frost, 500);
    vitals.resistances.insert(DamageType::Fire, -500);
    (vitals, Poise::new(100, 2), StatusSet::default())
}

#[test]
fn sword_hits_stagger_the_wolf_on_the_third_swing() {
    let (mut vitals, mut poise, _) = frost_wolf();
    let mut staggers = Vec::new();
    for _ in 0..3 {
        vitals.take_damage(DamageType::Physical, 30);
        staggers.push(poise.absorb(40, 45));
    }
    assert_eq!(staggers, vec![false, false, true]);
    assert_eq!(vitals.health, 110);
    assert!(poise.is_staggered());
}

#[test]
fn fire_burns_hotter_than_frost_on_a_frost_wolf() {
    let burn = |damage_type| {
        let (mut vitals, _, mut statuses) = frost_wolf();
        statuses.apply(StatusEffect {
            damage_type,
            damage_per_tick: 10,
            ticks_left: 6,
        });
        (0..6).for_each(|_| {
            statuses.tick(&mut vitals);
        });
        200 - vitals.health
    };
    assert_eq!(burn(DamageType::Fire), 90);
    assert_eq!(burn(DamageType::Frost), 30);
}

#[test]
fn staggered_wolf_dies_to_a_burst_of_fire() {
    let (mut vitals, mut poise, mut statuses) = frost_wolf();
    poise.absorb(100, 45);
    statuses.apply(StatusEffect {
        damage_type: DamageType::Fire,
        damage_per_tick: 20,
        ticks_left: 10,
    });
    let mut ticks = 0;
    while !vitals.is_dead() && ticks < 50 {
        statuses.tick(&mut vitals);
        poise.tick();
        ticks += 1;
    }
    assert!(vitals.is_dead());
    assert_eq!(ticks, 7);
    assert!(poise.is_staggered());
}

#[test]
fn swing_only_hits_targets_inside_its_arc() {
    let swing = Circle {
        x: 0,
        y: 0,
        radius: 1_500,
    };
    let near = Circle {
        x: 1_800,
        y: 0,
        radius: 400,
    };
    let far = Circle {
        x: 4_000,
        y: 0,
        radius: 400,
    };
    assert!(swing.overlaps(&near));
    assert!(!swing.overlaps(&far));
}
