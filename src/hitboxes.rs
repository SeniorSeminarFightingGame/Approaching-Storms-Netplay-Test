use bevy::prelude::*;

use super::{BoxCords, GameDirection, Materials};

pub struct HitboxSpawnEvent {
    pub position: BoxCords,
}

#[derive(Component)]
struct BoxBundle{
    topLeftX: f32,
    topLeftY: f32,
    bottomRightX: f32,
    bottomRightY: f32,
    visibility: Visibility,
}

fn setup_hitbox_hidden(
    mut commands: Commands,
) {
    commands.spawn((
        BoxBundle {
            topLeftX: 1.,
            topLeftY: 1.,
            bottomRightX: 0.,
            bottomRightY: 0.,
            visibility: Visibility {
                is_visible: true,
            },
            //..Default::default()
        },
    ));
}