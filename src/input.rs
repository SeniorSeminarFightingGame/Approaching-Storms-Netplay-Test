use bevy::prelude::*;
use bevy_ggrs::ggrs;

// Our input needs to be encoded to the u8 we defined in the GgrsConfig
// type and handed over to GGRS.
// Define some bit mask constants to signify what bit means what:
const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_LP: u8 = 1 << 4;
const INPUT_HP: u8 = 1 << 5;
const INPUT_LK: u8 = 1 << 6;
const INPUT_HK: u8 = 1 << 7;

// move the input sampling from move_player into a special input 
// system. This system need to return the same type we defined in our 
// GgrsConfig type, a u8.
pub fn input(_: In<ggrs::PlayerHandle>, keys: Res<Input<KeyCode>>) -> u8 {
    let mut input = 0u8;

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        input |= INPUT_LEFT
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        input |= INPUT_RIGHT;
    }
    if keys.just_pressed(KeyCode::O) {
        input |= INPUT_LP;
    }
    if keys.just_pressed(KeyCode::P) {
        input |= INPUT_HP;
    }
    if keys.just_pressed(KeyCode::L) {
        input |= INPUT_LK;
    }
    if keys.just_pressed(KeyCode::Colon) {
        input |= INPUT_HK;
    }

    input
}

// makes player move on keyboard / samples the keyboard and moves 
// any objects with the Player marker component in the given direction
// / convert the low-level input format to a direction
pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if input & INPUT_UP != 0 {
        direction.y += 1.;
        
        // STEP 09 jumping code:
        // if keyboard_input.just_pressed(KeyCode::W) && !jumper.is_jumping {
            //velocity.linvel = Vec2::new(0., jumper.jump_impulse).into();
            //jumper.is_jumping = true
        //}

    }

    if input & INPUT_DOWN != 0 {
        direction.y -= 1.;
    }

    if input & INPUT_RIGHT != 0 {
        direction.x += 1.;
        // STEP 09 jumping code:
        //if keyboard_input.pressed(KeyCode::D) && !jumper.is_jumping {
            //velocity.linvel = Vec2::new(player.speed, velocity.linvel.y).into();
            //player.facing_direction = GameDirection::Right
            //if keyboard_input.just_pressed(KeyCode::W) {
                //velocity.linvel = Vec2::new(player.speed, jumper.jump_impulse).into();
                //jumper.is_jumping = true
            //}
        //}
        
    }

    if input & INPUT_LEFT != 0 {
        direction.x -= 1.;
        // STEP 09 jumping code:
        //if keyboard_input.pressed(KeyCode::A) && !jumper.is_jumping {
            //velocity.linvel = Vec2::new(-player.speed, velocity.linvel.y).into();
            //if keyboard_input.just_pressed(KeyCode::W) {
                //velocity.linvel = Vec2::new(-player.speed, jumper.jump_impulse).into();
                //jumper.is_jumping = true
            //}
        //}

        // momentum control (prevents sliding)
        // if (keyboard_input.just_released(KeyCode::A) || keyboard_input.just_released(KeyCode::D)) && !jumper.is_jumping {
            // velocity.linvel = Vec2::new(0., velocity.linvel.y).into();
        // }

    }

    direction.normalize_or_zero()
}

// check for the fire button
pub fn fire(input: u8) -> bool {
    input & INPUT_LP != 0
}