use bevy::{math::Vec3Swizzles, prelude::*, render::camera::ScalingMode, tasks::IoTaskPool};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, *};
use components::*;
use input::*;
use matchbox_socket::WebRtcSocket;
use bevy_rapier2d::prelude::*; // floor and gravity
//use super::components::{Jumper, Materials, Player}; // inserting Jumper

mod components;
mod input;

// store the matchbox socket somewhere: it's accessible from multiple 
// systems, so create a new resource for things related to the current
// match/session.

#[derive(Resource)]
struct Session {
    socket: Option<WebRtcSocket>,
}

// A generic type parameter / This struct implements a trait that 
// tells GGRS about what kind of types our game uses
struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

// With bevy_asset_loader all assets are loaded in a special loading 
// state, and then it continues to the actual game state. So let’s 
// create some states:
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Matchmaking,
    InGame,
}

// resource so we have some way to tell which player is the local 
// player so we know what player the camera should follow
#[derive(Resource)]
struct LocalPlayerHandle(usize);

// Our input needs to be encoded to the u8 we defined in the GgrsConfig
// type and handed over to GGRS.
// Define some bit mask constants to signify what bit means what:
//const INPUT_UP: u8 = 1 << 0;
//const INPUT_DOWN: u8 = 1 << 1;
//const INPUT_LEFT: u8 = 1 << 2;
//const INPUT_RIGHT: u8 = 1 << 3;
//const INPUT_FIRE: u8 = 1 << 4;

fn main() {
    let mut app = App::new();

    GGRSPlugin::<GgrsConfig>::new() //tells our builder about our config and input system.
        .with_input_system(input)
        .with_rollback_schedule( // about all the systems that are affected by rollback
            Schedule::default().with_stage(
                "ROLLBACK_STAGE",
                SystemStage::single_threaded()
                    .with_system(move_players)
                    .with_system(reload_bullet)
                    .with_system(fire_bullets.after(move_players).after(reload_bullet)) // add our fire_bullets to our rollback stage / added explicit ordering to our rollback systems to make it deterministic
                    .with_system(move_bullet) // move the bullets to the right every frame after adding it to our rollback frame
                    .with_system(kill_players.after(move_bullet).after(move_players)) // kill players at the very end. That ensures the player is destroyed as close as possible to the detection (commands are executed at the end of the stage). That way they’re destroyed before they get the chance to take more actions
                    .with_system(spawn_floor), //adds our spawn_floor to the rollback stage
            ),
        )
        .register_rollback_component::<Transform>() // register the types we are interested in rolling back
        .register_rollback_component::<BulletReady>() // register BulletReady as a rollback type
        .register_rollback_component::<MoveDir>() // register MoveDir as a rollback type
        // Step 07 (not finished): register Jumper as a rollback component, then add to player controls
        //.register_rollback_component::<Jumper>()
        .build(&mut app);

    app.add_state(GameState::AssetLoading) //initialize GameState states
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .with_collection::<ImageAssets>()
                .continue_to_state(GameState::Matchmaking), // Continue Matchmaking state after loading
        )
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        // .insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities) // see types of gotchas through this special resource that was used to figure out the ordering between reloading & firing
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                // fill the entire browser window
                fit_canvas_to_parent: true,
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default()) // register rapier plugin with our application
        .add_system_set( // divide our systems into system sets for the appropriate states
            SystemSet::on_enter(GameState::Matchmaking)
                .with_system(start_matchbox_socket) // adds the start_matchbox_socket system
                .with_system(setup), //setup system that initializes a camera and a player sprite
        )
        .add_system_set(SystemSet::on_update(GameState::Matchmaking).with_system(wait_for_players)) // adds the wait_for_players system
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_players)) //adds the spawn_player system
        .add_system_set(SystemSet::on_update(GameState::InGame).with_system(camera_follow)) // adds the camera_follow system
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(spawn_floor))
        .run();
}

// adding a constant that defines our map width and height
const MAP_SIZE: i32 = 41;
const GRID_WIDTH: f32 = 0.05;

// An asset collection for our images
#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "bullet.png")]
    bullet: Handle<Image>,
}

// add struct RigidBodyBundle

// code for setup system that initializes a camera and a player sprite
fn setup(mut commands: Commands) {
    // spawn the sprites
    // Horizontal lines
    for i in 0..=MAP_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(MAP_SIZE as f32, GRID_WIDTH)),
                ..default()
            },
            ..default()
        });
    }

    // Vertical lines
    for i in 0..=MAP_SIZE {
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, MAP_SIZE as f32)),
                ..default()
            },
            ..default()
        });
    }

    /* STEP 01 */
    let rigid_body = RigidBody::default();
    let collider = Collider::cuboid(1.0, 2.0);


    commands.spawn(rigid_body); //RigidBodyBundle
    commands.spawn(collider); //ColliderBundle

    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.); //fixed vertical size of 10
    commands.spawn(camera_bundle);
}

// adds players
fn spawn_players(mut commands: Commands, mut rip: ResMut<RollbackIdProvider>) {
    info!("Spawning players");

    // Player 1
    commands.spawn((
        Player { handle: 0, speed: 4., facing_direction: GameDirection::Right, hp: 100.,}, // adds player component to player entity / added speed, facing_direction, and hp (STEP 05)
        Jumper {jump_impulse: 14., is_jumping: false,},
        BulletReady(true), //add BulletReady rollback type when we spawn player
        MoveDir(-Vec2::X), // keep track of the player direction
        Rollback::new(rip.next_id()), // adds rollback component to player entity

        /*
        RigidBodyBundle {
            mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
            activation: RigidBodyActivation::cannot_sleep(),
            ccd: RigidBodyCcd { ccd_enabled: true, ..Default::default() },
            ..Default::default()
        },

        ColliderBundle {
            shape: ColliderShape::cuboid(0.5, 0.5),
            flags: ColliderFlags {
            active_events: ActiveEvents::CONTACT_EVENTS,
            ..Default::default()
            },
            ..Default::default()
        },
        */

        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 0., 100.)), // Instead of moving the background forward, we’ll move the players closer to the camera because of z
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()
        },
        
    )
)
        // STEP 02: Rigid-Bodies & Colliders
        .insert(TransformBundle::from(Transform::from_xyz(0.0, 5.0, 0.0)))
        .insert(Velocity {
            linvel: Vec2::new(1.0, 2.0),
            angvel: 0.2
        })

        // STEP 04: gravity and control
        //.insert(Jumper {
           // jump_impulse: 14.,
           // is_jumping: false,
        //})

        .insert(GravityScale(0.5))
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())

        .insert(Sensor)
        .insert(TransformBundle::from(Transform::from_xyz(2.0, 0.0, 0.0)))
        .insert(Friction::coefficient(0.7))
        .insert(Restitution::coefficient(0.3))
        .insert(ColliderMassProperties::Density(2.0))

;

    // Player 2
    commands.spawn((
        Player { handle: 1, speed: 4., facing_direction: GameDirection::Left, hp: 100., }, // adds player component to player entity
        Jumper {jump_impulse: 14., is_jumping: false,},
        BulletReady(true), //add BulletReady rollback type when we spawn player
        MoveDir(Vec2::X), // keep track of the player direction
        Rollback::new(rip.next_id()), // adds rollback component to player entity
        SpriteBundle {
            transform: Transform::from_translation(Vec3::new(2., 0., 100.)), // Instead of moving the background forward, we’ll move the players closer to the camera because of z
            sprite: Sprite {
                color: Color::rgb(0., 0.4, 0.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            ..default()
        },
    ))
    //.insert(rigid_body)
    //.insert(collider)
    //.insert(RigidBodyPositionSync::Discrete)
    ;
}

// A system that creates the socket which connects to the Matchbox 
// server and establishes direct connections to other clients.
fn start_matchbox_socket(mut commands: Commands) {
    let room_url = "ws://127.0.0.1:3536/extreme_bevy?next=2";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    IoTaskPool::get().spawn(message_loop).detach();

    commands.insert_resource(Session {
        socket: Some(socket),
    });
}

// a system where we just wait until we've established a peer 
// connection and then log to the console so can make sure everything 
// works so far:
fn wait_for_players(
    mut commands: Commands,
    mut session: ResMut<Session>,
    mut state: ResMut<State<GameState>>, // make sure that we actually enter the InGame state in wait_for_players when we start the GGRS session
) {
    let Some(socket) = &mut session.socket else {
        // If there is no socket we've already started the game
        return;
    };

    // Check for new connections
    socket.accept_new_connections();
    let players = socket.players();

    let num_players = 2;
    if players.len() < num_players {
        return; // wait for more players
    }

    info!("All peers have joined, going in-game");

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players) // how many players will there be
        .with_input_delay(2); //input delay

    // adds players to the session, where we just need to assign a 
    // handle to each of them. We simply assign integers in the order 
    // that they arrive.
    for (i, player) in players.into_iter().enumerate() {
        if player == PlayerType::Local { // inserting the LocalPlayerHandle into wait_for_players
            commands.insert_resource(LocalPlayerHandle(i));
        }

        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = session.socket.take().unwrap();

    // start the GGRS session
    let ggrs_session = session_builder // creating a bevy_ggrs session using its SessionBuilder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(bevy_ggrs::Session::P2PSession(ggrs_session));
    state.set(GameState::InGame).unwrap();
}

// makes player move on keyboard / samples the keyboard and moves 
// any objects with the Player marker component in the given direction
// STEP 06: moving players
fn move_players(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut player_query: Query<(&mut Transform, &mut MoveDir, &Player, &mut Jumper, &mut Velocity,)>,
) {
    for (mut transform, mut move_direction, player, jumper, velocity) in player_query.iter_mut() { // Step 07: adding jumper / velocity component
        let (input, _) = inputs[player.handle];
        let direction = direction(input);

        if direction == Vec2::ZERO {
            continue;
        }

        move_direction.0 = direction;

         // STEP 08: Jumping / Velocity code (nothing here yet)

        // our player shouldn't be able to move out of the map
        let move_speed = 0.13;
        let move_delta = direction * move_speed;

        let old_pos = transform.translation.xy();
        let limit = Vec2::splat(MAP_SIZE as f32 / 2. - 0.5); // makes sure no players goes away from the map
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

// In the above code, We connect to the Matchbox server running on our
// machine, and ask to join the extreme_bevy?next=2 room. 
// The first part before the question mark, is an id. This means you 
// can use the same matchbox instance for different scopes. This could
// be useful if you want to use the same matchbox server for several 
// types of games, or different game modes, or different versions of 
// the game etc.
//
// The second part, next=2 is a special type of room that connects 
// pairs of peers together. The first two peers to connect will be 
// paired, and then the third and fourth will be paired and so on. 
// It's perfect for just testing that we can get 2-players up and 
// running.
//
// Next, we call WebRtcSocket::new, to create a new GGRS-compatible 
// socket type. It returns both a socket, and a message loop future 
// that needs to be awaited in order to process network events. We 
// await it using Bevy's task system.
//
// Finally, we create our Session resource and initialize it with our 
// newly created socket so we can access the socket from other systems.

fn reload_bullet(
    inputs: Res<PlayerInputs<GgrsConfig>>,
    mut query: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in query.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

// When a player presses a button, we want to spawn a sprite with the 
// bullet image. Let’s make a new system for this
fn fire_bullets(
    mut commands: Commands,
    inputs: Res<PlayerInputs<GgrsConfig>>,
    images: Res<ImageAssets>,
    mut player_query: Query<(&Transform, &Player, &mut BulletReady, &MoveDir)>,
    mut rip: ResMut<RollbackIdProvider>, // make sure our bullets are rolled back
) {
    for (transform, player, mut bullet_ready, move_dir) in player_query.iter_mut() {
        // spawn our bullet sprite
        let (input, _) = inputs[player.handle];
        // check whether a bullet is ready
        if fire(input) && bullet_ready.0 {
            let player_pos = transform.translation.xy(); // moving the bullet a little bit away from the player when spawning it
            let pos = player_pos + move_dir.0 * PLAYER_RADIUS + BULLET_RADIUS; // moving the bullet a little bit away from the player when spawning it
            commands.spawn((
                Bullet, // now that we have a player direction, we can copy it to our bullets in fire_bullets
                Rollback::new(rip.next_id()), // make sure our bullets are rolled back
                *move_dir,
                SpriteBundle {
                    transform: Transform::from_translation(pos.extend(200.))
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, move_dir.0)), // moving the bullet a little bit away from the player when spawning it
                    texture: images.bullet.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(0.3, 0.1)),
                        ..default()
                    },
                    ..default()
                },
            ));
            bullet_ready.0 = false; // set bullet ready to false when we actually fire
        }
    }
}

// write the system
fn move_bullet(mut query: Query<(&mut Transform, &MoveDir), With<Bullet>>) {
    for (mut transform, dir) in query.iter_mut() {
        let delta = (dir.0 * 0.35).extend(0.);
        transform.translation += delta;
    }
}

const PLAYER_RADIUS: f32 = 0.5;
const BULLET_RADIUS: f32 = 0.025;


// A player should die if the bullet overlaps the player. In other 
// words, if the distance between their centers are smaller than the 
// sum of their radii
fn kill_players(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Bullet>)>,
    bullet_query: Query<&Transform, With<Bullet>>,
) {
    for (player, player_transform) in player_query.iter() {
        for bullet_transform in bullet_query.iter() {
            let distance = Vec2::distance(
                player_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < PLAYER_RADIUS + BULLET_RADIUS {
                commands.entity(player).despawn_recursive();
            }
        }
    }
}

// declare our inputs
fn camera_follow(
    player_handle: Option<Res<LocalPlayerHandle>>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    // If there is no player handle yet, we simply return early
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    // loop through and find the player position
    for (player, player_transform) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

/* STEP 03: Make Floor ( let mut rigid_body = RigidBody::default();
    let mut collider = Collider::cuboid(1.0, 2.0);) */
fn spawn_floor(mut commands: Commands) {
// mut materials: ResMut<Assets<ColorMaterial>>

    commands
    .spawn(Collider::cuboid(500.0, 50.0))
    .insert(Name::new("Ground"))
    .insert(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));
    /* 
    let width = 10.;
    let height = 1.;
    
    let rigid_body02 = RigidBody {
        position: Vec2::new(0.0, -2.).into(),
        body_type: RigidBody::Fixed,
        ..Default::default()
    };
    let collider02 = Collider {
        shape: ColliderShape::cuboid(width / 2., height / 2.),
        ..Default::default()
    };
    commands
        .spawn(SpriteBundle {
            material: materials.add(Color::rgb(0.7, 0.7, 0.7).into()),
            sprite: Sprite::new(Vec2::new(width, height)),
            ..Default::default()
        })
        .insert(rigid_body02)
        .insert(collider02)
        .insert(RigidBodyPositionSync::Discrete);
    */
}