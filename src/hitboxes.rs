use bevy::{math::Vec3Swizzles,prelude::*};
use crate::components::*;
use bevy_ggrs::ggrs;

use super::{GameDirection, Materials};

pub struct HitboxSpawnEvent {
    
}

pub fn make_hitbox(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &Player)>,
    mut materials: Materials,
    newWidth: f32,
    newHeight: f32,
    xOffset: f32,
    yOffset: f32,
    newDamage: f32,
    newChip: f32,
    newHitstun: i32,
    newBlockstun: i32
) {
    for (transform, player) in player_query.iter_mut() {
        let player_pos = transform.translation.xy();
        commands.spawn((
            Hitbox {
                width: newWidth,
                height: newHeight,
                position: player_pos + Vec2::new(xOffset, yOffset),
                damage: newDamage,
                chip: newChip,
                hitstun: newHitstun,
                blockstun: newBlockstun,
                visibility: Visibility {
                    is_visible: true,
                },
                ..default()
            },
            /*Sprite {
                color: materials.hitbox_material.clone(),
                custom_size: Some(Vec2::new(newWidth, newHeight)),
                ..Default::default()
            },*/
        ));
    }
}

pub fn make_grabbox(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &Player)>,
    mut materials: Materials,
    newWidth: f32,
    newHeight: f32,
    xOffset: f32,
    yOffset: f32
) {
    for (transform, player) in player_query.iter_mut() {
        let player_pos = transform.translation.xy();
        commands.spawn((
            Grabbox {
                width: newWidth,
                height: newHeight,
                position: player_pos + Vec2::new(xOffset, yOffset),
                visibility: Visibility {
                    is_visible: true,
                },
                ..default()
            },
            /*sprite = SpriteBundle {
                material: materials.hitbox_material.clone(),
                sprite: Sprite::new(Vec2::new(newWidth, newHeight)),
                ..Default::default()
            };*/
        ));
    }
}