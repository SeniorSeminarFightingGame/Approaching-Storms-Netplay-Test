use bevy::prelude::*;

pub struct Materials {
    pub player_material: Handle<ColorMaterial>,
    pub floor_material: Handle<ColorMaterial>,
    pub hitbox_material: Handle<ColorMaterial>,
}

// platformer additions
#[derive(Copy, Clone)]
pub enum GameDirection {
    Left,
    Right,
}

// adds the Player marker component
#[derive(Component)]
pub struct Player {
    pub handle: usize,

    // platformer additions
    pub speed: f32,
    pub facing_direction: GameDirection,
    pub hp: f32,
}

#[derive(Component)]
pub struct Jumper {
    pub jump_impulse: f32,
    pub is_jumping: bool,
}

#[derive(Component)]
pub struct PlayerStates {
    pub Idle: f32,
    pub Walk: f32,
    pub WalkB: f32,
    pub Crouch: f32,
    pub Jump: f32,
    pub Dash: f32,
    pub DashB: f32,
    pub Block: f32,
    pub BlockC: f32,
    pub Hitstun: f32,
    pub HitstunC: f32,
    pub HitstunA: f32,
    pub Launch: f32,
    pub Falldown: f32,
    pub Knockdown: f32,
    pub Wallbounce: f32,
    pub Wakeup: f32,
    pub WakeupQ: f32,
    pub Defeat: f32,
    pub PunchL: f32,
    pub PunchH: f32,
    pub KickL: f32,
    pub KickH: f32,
    pub PunchLC: f32,
    pub PunchHC: f32,
    pub KickLC: f32,
    pub KickHC: f32,
    pub PunchLA: f32,
    pub PunchHA: f32,
    pub KickLA: f32,
    pub KickHA: f32,
    pub Command: f32,
    pub Grab: f32,
    pub ThrowF: f32,
    pub ThrowB: f32,
    pub Special: f32,
}

// add a new component that keeps track of whether a bullet is ready 
// to be fired. This way we can extend it to also handle other kinds 
// of cooldowns if necessary
#[derive(Component, Reflect, Default)]
pub struct BulletReady(pub bool);

// before we can write the system, we need to know which components 
// are actually bullets
#[derive(Component, Reflect, Default)]
pub struct Bullet;

// To spawn bullets in the correct direction, we’ll start by spawning 
// them with the right orientation, we will do that by setting their 
// rotation to the direction the player is currently facing. Since we 
// will use sprites for representing player direction, we won’t 
// actually rotate the players themselves, but what we’ll do is 
// instead keep track of the player direction in a special MoveDir 
// component. It will be a newtype of Vec2.
#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct MoveDir(pub Vec2);

#[derive(Component, Default)]
pub struct BoxCords{
    topLeftX: f32,
    topLeftY: f32,
    bottomRightX: f32,
    bottomRightY: f32,
}