use bevy::color::palettes;
use bevy::image::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_magic_light_2d::gi::render_layer::ALL_LAYERS;
use bevy_magic_light_2d::prelude::*;
use rand::prelude::*;

pub const TILE_SIZE: f32 = 16.0;
pub const SPRITE_SCALE: f32 = 4.0;
pub const Z_BASE_FLOOR: f32 = 100.0; // Base z-coordinate for 2D layers.
pub const Z_BASE_OBJECTS: f32 = 200.0; // Ground object sprites.
pub const SCREEN_SIZE: (f32, f32) = (1280.0, 720.0);
pub const CAMERA_SCALE: f32 = 1.0;
pub const CAMERA_SCALE_BOUNDS: (f32, f32) = (1., 20.);
pub const CAMERA_ZOOM_SPEED: f32 = 3.;

// Misc components.
#[derive(Component)]
pub struct MouseLight;
#[derive(Component)]
pub struct Movable;

fn main()
{
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::srgba_u8(0, 0, 0, 0)))
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes_override: Some(true),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: SCREEN_SIZE.into(),
                        title: "Bevy Magic Light 2D: Krypta Example".into(),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        mag_filter: ImageFilterMode::Nearest,
                        min_filter: ImageFilterMode::Nearest,
                        ..default()
                    },
                }),
            BevyMagicLight2DPlugin,
            ResourceInspectorPlugin::<BevyMagicLight2DSettings>::new(),
        ))
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            },
            ..default()
        })
        .register_type::<LightOccluder2D>()
        .register_type::<OmniLightSource2D>()
        .register_type::<SkylightMask2D>()
        .register_type::<SkylightLight2D>()
        .register_type::<BevyMagicLight2DSettings>()
        .register_type::<LightPassParams>()
        .add_systems(Startup, setup.after(setup_post_processing_camera))
        .add_systems(Update, (system_move_camera, system_camera_zoom))
        .add_systems(Update, system_control_mouse_light.after(system_move_camera))
        .run();
}

#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    camera_targets: Res<CameraTargets>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
)
{
    // Utility functions to compute Z coordinate for floor and ground objects.
    let get_floor_z = |y| -> f32 { Z_BASE_FLOOR - y / SCREEN_SIZE.1 };
    let get_object_z = |y| -> f32 { Z_BASE_OBJECTS - y / SCREEN_SIZE.1 };

    // Maze map. 1 represents wall.
    let walls_info: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    // Generate square occluders from walls_info.
    let block_size = Vec2::splat(TILE_SIZE * SPRITE_SCALE);
    let center_offset =
        Vec2::new(-1024.0, 1024.0) / 2.0 + block_size / 2.0 - Vec2::new(0.0, block_size.y);

    let get_block_translation = |i: usize, j: usize| {
        center_offset + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y)
    };

    let block_mesh = meshes.add(Mesh::from(Rectangle::default()));
    let mut walls = vec![];

    // Load floor tiles.
    let floor_atlas_rows = 4;
    let floor_atlas_cols = 4;
    let floor_atlas_size = UVec2::new(16, 16);
    let floor_image = asset_server.load("art/atlas_floor.png");
    let floor_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        floor_atlas_size,
        floor_atlas_cols,
        floor_atlas_rows,
        None,
        None,
    ));

    // Load wall tiles.
    let wall_atlas_rows = 5;
    let wall_atlas_cols = 6;
    let wall_atlas_size = UVec2::new(16, 16);
    let wall_image = asset_server.load("art/atlas_wall.png");
    let wall_atlas = texture_atlases.add(TextureAtlasLayout::from_grid(
        wall_atlas_size,
        wall_atlas_cols,
        wall_atlas_rows,
        None,
        None,
    ));

    // Load decoration sprites.
    let decorations_image = asset_server.load("art/atlas_decoration.png");

    // Spawn floor tiles.
    let mut rng = rand::rng();
    let mut floor_tiles = vec![];
    for (i, row) in walls_info.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {
            let xy = get_block_translation(i, j);
            let z = get_floor_z(xy.y);
            let id = rng.random_range(0..(floor_atlas_cols * floor_atlas_rows));

            floor_tiles.push(
                commands
                    .spawn((
                        Transform {
                            translation: Vec3::new(xy.x, xy.y, z),
                            scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                            ..default()
                        },
                        Sprite::from_atlas_image(
                            floor_image.clone(),
                            TextureAtlas {
                                layout: floor_atlas.clone(),
                                index:  id as usize,
                            },
                        ),
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR))
                    .id(),
            );
        }
    }

    commands
        .spawn(Name::new("floor_tiles"))
        .insert((Transform::default(), Visibility::default()))
        .add_children(&floor_tiles);

    let maze_rows = walls_info.len() as i32;
    let maze_cols = walls_info[0].len() as i32;

    // Get wall value clamping to edge.
    let get_wall_safe = |r: i32, c: i32, offset: (i32, i32)| {
        let r1 = r + offset.0;
        let c1 = c + offset.1;
        if r1 < 0 || r1 >= maze_rows {
            return 1;
        }
        if c1 < 0 || c1 >= maze_cols {
            return 1;
        }
        walls_info[r1 as usize][c1 as usize]
    };

    // Get atlas sprite index for wall.
    let get_wall_sprite_index = |r: usize, c: usize| {
        let r = r as i32;
        let c = c as i32;

        let w_up = get_wall_safe(r, c, (-1, 0));
        let w_down = get_wall_safe(r, c, (1, 0));
        let w_left = get_wall_safe(r, c, (0, -1));
        let w_right = get_wall_safe(r, c, (0, 1));

        let total_walls = w_up + w_down + w_left + w_right;

        if total_walls == 4 {
            return wall_atlas_cols * 0 + 0;
        }

        if total_walls == 3 {
            if w_up == 0 {
                return wall_atlas_cols * 1 + 0;
            }
            if w_left == 0 {
                return wall_atlas_cols * 1 + 1;
            }
            if w_down == 0 {
                return wall_atlas_cols * 1 + 2;
            }
            if w_right == 0 {
                return wall_atlas_cols * 1 + 3;
            }
        }

        if total_walls == 2 {
            if w_left == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 0;
            }

            if w_up == 1 && w_down == 1 {
                return wall_atlas_cols * 2 + 1;
            }

            if w_up == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 2;
            }

            if w_down == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 3;
            }

            if w_up == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 4;
            }

            if w_down == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 5;
            }
        }

        if total_walls == 1 {
            if w_left == 1 {
                return wall_atlas_cols * 3 + 0;
            }
            if w_down == 1 {
                return wall_atlas_cols * 3 + 1;
            }
            if w_up == 1 {
                return wall_atlas_cols * 3 + 2;
            }
            if w_right == 1 {
                return wall_atlas_cols * 3 + 3;
            }
        }

        wall_atlas_cols * 4 + 0
    };

    // Add walls with occluder component.
    let occluder_data = LightOccluder2D {
        h_size: block_size / 2.0,
    };
    for (i, row) in walls_info.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if *cell == 1 {
                let xy = get_block_translation(i, j);
                let z = get_object_z(xy.y);
                let id = get_wall_sprite_index(i, j);

                walls.push(
                    commands
                        .spawn((
                            Transform {
                                translation: Vec3::new(xy.x, xy.y, z),
                                scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                                ..default()
                            },
                            Sprite::from_atlas_image(
                                wall_image.clone(),
                                TextureAtlas {
                                    layout: wall_atlas.clone(),
                                    index:  id as usize,
                                },
                            ),
                        ))
                        .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS))
                        .insert(occluder_data)
                        .id(),
                );
            }
        }
    }
    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("walls"))
        .add_children(&walls);

    // Add decorations.
    let mut decorations = vec![];
    {
        let mut decorations_atlas = TextureAtlasLayout::new_empty(UVec2::new(256, 256));

        let candle_rect_1 = decorations_atlas.add_texture(URect {
            min: UVec2::new(0, 0),
            max: UVec2::new(16, 16),
        });
        let candle_rect_2 = decorations_atlas.add_texture(URect {
            min: UVec2::new(16, 0),
            max: UVec2::new(32, 16),
        });
        let candle_rect_3 = decorations_atlas.add_texture(URect {
            min: UVec2::new(32, 0),
            max: UVec2::new(48, 16),
        });
        let candle_rect_4 = decorations_atlas.add_texture(URect {
            min: UVec2::new(48, 0),
            max: UVec2::new(64, 16),
        });
        let tomb_rect_1 = decorations_atlas.add_texture(URect {
            min: UVec2::new(32, 16),
            max: UVec2::new(80, 48),
        });
        let sewerage_rect_1 = decorations_atlas.add_texture(URect {
            min: UVec2::new(0, 16),
            max: UVec2::new(32, 48),
        });

        let texture_atlas_handle = texture_atlases.add(decorations_atlas);

        // Candle 1.
        {
            let x = 100.0;
            let y = -388.5;

            decorations.push(
                commands
                    .spawn((
                        Visibility::default(),
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(180, 180, 180),
                            image: decorations_image.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  candle_rect_1,
                            }),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::splat(2.0),
                    })
                    .insert(Name::new("candle_1"))
                    .id(),
            );
        }

        // Candle 2.
        {
            let x = -32.1;
            let y = -384.2;

            decorations.push(
                commands
                    .spawn((
                        Visibility::default(),
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(180, 180, 180),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  candle_rect_2,
                            }),
                            image: decorations_image.clone(),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::splat(2.0),
                    })
                    .insert(Name::new("candle_2"))
                    .id(),
            );
        }

        // Candle 3.
        {
            let x = -351.5;
            let y = -126.0;

            decorations.push(
                commands
                    .spawn((
                        Visibility::default(),
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(180, 180, 180),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  candle_rect_3,
                            }),
                            image: decorations_image.clone(),

                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::splat(2.0),
                    })
                    .insert(Name::new("candle_3"))
                    .id(),
            );
        }

        // Candle 4.
        {
            let x = 413.0;
            let y = -124.6;

            decorations.push(
                commands
                    .spawn((
                        Visibility::default(),
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(180, 180, 180),
                            image: decorations_image.clone(),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  candle_rect_4,
                            }),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::splat(2.0),
                    })
                    .insert(Name::new("candle_4"))
                    .id(),
            );
        }

        // Tomb 1.
        {
            let x = 31.5;
            let y = -220.0;
            decorations.push(
                commands
                    .spawn((
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(255, 255, 255),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  tomb_rect_1,
                            }),
                            image: decorations_image.clone(),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::new(72.8, 31.0),
                    })
                    .insert(Name::new("tomb_1"))
                    .id(),
            );
        }

        // Tomb 1.
        {
            let x = 300.5;
            let y = -500.0;
            decorations.push(
                commands
                    .spawn((
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(255, 255, 255),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index:  tomb_rect_1,
                            }),
                            image: decorations_image.clone(),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                    .insert(LightOccluder2D {
                        h_size: Vec2::new(72.8, 31.0),
                    })
                    .insert(Name::new("tomb_1"))
                    .id(),
            );
        }

        // Sewerage 1.
        {
            let x = 31.5;
            let y = -38.5;

            decorations.push(
                commands
                    .spawn((
                        Transform {
                            translation: Vec3::new(x, y, get_object_z(y)),
                            scale: Vec2::splat(4.0).extend(0.0),
                            ..default()
                        },
                        Sprite {
                            color: Color::srgb_u8(255, 255, 255),
                            texture_atlas: Some(TextureAtlas {
                                layout: texture_atlas_handle.clone(),
                                index: sewerage_rect_1,
                                ..default()
                            }),
                            image: decorations_image.clone(),
                            ..default()
                        },
                    ))
                    .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR)) // Add to floor
                    .insert(Name::new("sewerage_1"))
                    .id(),
            );
        }
    }
    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("decorations"))
        .add_children(&decorations);

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light = |cmd: &mut Commands,
                           x: f32,
                           y: f32,
                           name: &'static str,
                           light_source: OmniLightSource2D| {
            cmd.spawn(Name::new(name))
                .insert(light_source)
                .insert((
                    Visibility::default(),
                    Transform {
                        translation: Vec3::new(x, y, 0.0),
                        ..default()
                    },
                ))
                .insert(RenderLayers::from_layers(ALL_LAYERS))
                .id()
        };

        let base = OmniLightSource2D {
            falloff: Vec3::new(50.0, 20.0, 0.05),
            intensity: 10.0,
            ..default()
        };
        lights.push(spawn_light(
            &mut commands,
            90.667,
            -393.8,
            "outdoor_krypta_torch_1",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::srgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -36.000,
            -393.8,
            "outdoor_krypta_torch_2",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::srgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            230.9,
            -284.6,
            "indoor_krypta_light_1",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::srgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -163.5,
            -292.7,
            "indoor_krypta_light_2",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::srgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -352.000,
            -131.2,
            "outdoor_krypta_torch_3",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::srgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            410.667,
            -141.8,
            "outdoor_krypta_torch_4",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::srgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            28.0,
            -34.0,
            "indoor_krypta_ghost_1",
            OmniLightSource2D {
                intensity: 0.8,
                color: Color::srgb_u8(6, 53, 6),
                jitter_intensity: 0.2,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            31.392,
            -168.3,
            "indoor_krypta_tomb_1",
            OmniLightSource2D {
                intensity: 0.4,
                color: Color::srgb_u8(252, 182, 182),
                jitter_intensity: 0.05,
                jitter_translation: 4.7,
                ..base
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            40.0,
            -1163.2,
            "outdoor_light_9",
            OmniLightSource2D {
                intensity:          1.2,
                falloff:            Vec3::new(50.0, 40.0, 0.03),
                color:              Color::srgb_u8(0, 206, 94),
                jitter_intensity:   0.7,
                jitter_translation: 3.0,
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            182.3,
            -1210.0,
            "outdoor_light_10",
            OmniLightSource2D {
                intensity:          1.2,
                falloff:            Vec3::new(50.0, 40.0, 0.03),
                color:              Color::srgb_u8(0, 206, 94),
                jitter_intensity:   0.7,
                jitter_translation: 3.0,
            },
        ));
    }
    commands
        .spawn((Transform::default(), Visibility::default()))
        .insert(Name::new("lights"))
        .add_children(&lights);

    // Add roofs.
    commands
        .spawn((
            Visibility::default(),
            Transform::from_translation(Vec3::new(30.0, -180.0, 0.0)),
        ))
        .insert(Name::new("skylight_mask_1"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(430.0, 330.0),
        });
    commands
        .spawn((
            Visibility::default(),
            Transform::from_translation(Vec3::new(101.6, -989.4, 0.0)),
        ))
        .insert(Name::new("skylight_mask_2"))
        .insert(SkylightMask2D {
            h_size: Vec2::new(163.3, 156.1),
        });

    // Add skylight light.
    commands.spawn((
        SkylightLight2D {
            color:     Color::srgb_u8(93, 158, 179),
            intensity: 0.025,
        },
        Name::new("global_skylight"),
    ));

    // Add light source.
    commands
        .spawn((
            Mesh2d(block_mesh.into()),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(palettes::basic::YELLOW))),
            Transform {
                translation: Vec3::new(0.0, 0.0, 1000.0),
                scale: Vec3::splat(8.0),
                ..default()
            },
        ))
        .insert(Name::new("cursor_light"))
        .insert(OmniLightSource2D {
            intensity: 10.0,
            color: Color::srgb_u8(254, 100, 34),
            falloff: Vec3::new(50.0, 20.0, 0.05),
            ..default()
        })
        .insert(RenderLayers::from_layers(ALL_LAYERS))
        .insert(MouseLight);

    let projection = OrthographicProjection {
        scale: CAMERA_SCALE,
        near: -2000.0,
        far: 2000.0,
        ..OrthographicProjection::default_2d()
    };

    // Setup separate camera for floor, walls and objects.
    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.floor_target.clone()),
                ..default()
            },
            projection.clone(),
            Name::new("floors_target_camera"),
        ))
        .insert(SpriteCamera)
        .insert(FloorCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR));
    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.walls_target.clone()),
                ..default()
            },
            projection.clone(),
            Name::new("walls_target_camera"),
        ))
        .insert(SpriteCamera)
        .insert(WallsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS));
    commands
        .spawn((
            Camera2d,
            Camera {
                hdr: false,
                target: RenderTarget::Image(camera_targets.objects_target.clone()),
                ..default()
            },
            projection,
            Name::new("objects_targets_camera"),
        ))
        .insert(SpriteCamera)
        .insert(ObjectsCamera)
        .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS));
}

fn system_control_mouse_light(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    mut query_light: Query<(&mut Transform, &mut OmniLightSource2D), With<MouseLight>>,
    query_cameras: Query<(&Camera, &GlobalTransform), With<SpriteCamera>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
)
{
    let mut rng = rand::rng();

    // We only need to iter over first camera matched.
    let (camera, camera_transform) = query_cameras.iter().next().unwrap();
    let Ok(window) = window.get_single() else {
        return;
    };

    if let Some(screen_pos) = window.cursor_position() {
        let window_size = Vec2::new(window.width(), window.height());
        let mut mouse_ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;
        mouse_ndc = Vec2::new(mouse_ndc.x, -mouse_ndc.y);
        let ndc_to_world = camera_transform.compute_matrix() * camera.clip_from_view().inverse();
        let mouse_world = ndc_to_world.project_point3(mouse_ndc.extend(-1.0));

        let (mut mouse_transform, mut mouse_color) = query_light.single_mut();
        mouse_transform.translation = mouse_world.truncate().extend(1000.0);

        if mouse.just_pressed(MouseButton::Right) {
            mouse_color.color = Color::srgba(rng.random(), rng.random(), rng.random(), 1.0);
        }
        if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ShiftLeft) {
            commands
                .spawn((
                    Visibility::default(),
                    Transform {
                        translation: mouse_world.truncate().extend(0.0),
                        ..default()
                    },
                ))
                .insert(Name::new("point_light"))
                .insert(RenderLayers::from_layers(ALL_LAYERS))
                .insert(OmniLightSource2D {
                    jitter_intensity: 0.0,
                    jitter_translation: 0.0,
                    ..*mouse_color
                });
        }
    }
}

#[rustfmt::skip]
fn system_move_camera(
    mut camera_current: Local<Vec2>,
    mut camera_target:  Local<Vec2>,
    mut query_cameras:  Query<&mut Transform, With<SpriteCamera>>,
        keyboard:       Res<ButtonInput<KeyCode>>,
) {
    let speed = 18.0;
    if keyboard.pressed(KeyCode::KeyW) { camera_target.y += speed; }
    if keyboard.pressed(KeyCode::KeyS) { camera_target.y -= speed; }
    if keyboard.pressed(KeyCode::KeyA) { camera_target.x -= speed; }
    if keyboard.pressed(KeyCode::KeyD) { camera_target.x += speed; }

    // Smooth camera.
    let blend_ratio = 0.2;
    let movement = *camera_target - *camera_current;
    *camera_current += movement * blend_ratio;

    // Update all sprite cameras.
    for mut camera_transform in query_cameras.iter_mut() {
        camera_transform.translation.x = camera_current.x;
        camera_transform.translation.y = camera_current.y;
    }
}

fn system_camera_zoom(
    mut cameras: Query<&mut OrthographicProjection, With<SpriteCamera>>,
    time: Res<Time>,
    mut scroll_event_reader: EventReader<MouseWheel>,
)
{
    let mut projection_delta = 0.;

    for event in scroll_event_reader.read() {
        projection_delta += event.y * CAMERA_ZOOM_SPEED;
    }

    if projection_delta == 0. {
        return;
    }

    for mut camera in cameras.iter_mut() {
        camera.scale = (camera.scale - projection_delta * time.delta_secs())
            .clamp(CAMERA_SCALE_BOUNDS.0, CAMERA_SCALE_BOUNDS.1);
    }
}
