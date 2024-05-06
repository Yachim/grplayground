use std::{f32::consts::PI, vec};

use bevy::{math::vec3, prelude::*, render::{camera::ScalingMode, render_resource::{AsBindGroup, ShaderRef}}, sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle}};
use bevy_simple_text_input::{TextInputBundle, TextInputInactive, TextInputPlugin, TextInputValue};
use smooth_bevy_cameras::{controllers::unreal::{UnrealCameraBundle, UnrealCameraController, UnrealCameraPlugin}, LookTransform, LookTransformPlugin};

fn main() {
    App::new()
        .insert_resource(WindowData::default())
        .insert_resource(CamData::default())
        .insert_resource(SpacetimeParams::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "General Relativity Playground".to_string(),
                    ..default()
                }),
                ..default()
            }),
        ))
        .add_plugins(Material2dPlugin::<SchwarzschildMaterial>::default())
        .add_plugins(TextInputPlugin)
        .add_plugins((LookTransformPlugin, UnrealCameraPlugin::default()))
        .add_systems(Startup, setup)
        .add_systems(Update, (focus, update_window_data, update_material, update_camera_data, update_position_text, update_spacetime_params))
        .run();
}

const INPUT_BORDER_COLOR_ACTIVE: Color = Color::rgb(0.4, 0.4, 0.4);
const INPUT_BORDER_COLOR_INACTIVE: Color = Color::rgb(0.25, 0.25, 0.25);
const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const INPUT_BG_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const SIDEBAR_BG_COLOR: Color = Color::rgba(0.1, 0.1, 0.1, 0.9);

const FONT_PATH: &str = "fonts/noto_sans/static/NotoSans-Regular.ttf";

const NEWTON_CONSTANT: f32 = 6.67e-11;
const LIGHT_SPEED: f32 = 299_792_458.;

/* #region units */
fn length_to_si(val: f32, mass: f64) -> f64 {
    mass * ((val * NEWTON_CONSTANT / (LIGHT_SPEED * LIGHT_SPEED)) as f64)
}

fn time_to_geo(val: f32, mass: f64) -> f64 {
    (val * LIGHT_SPEED.powf(3.) / NEWTON_CONSTANT) as f64 / mass
}
/* #endregion */

/* #region shader */
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct SchwarzschildMaterial {
    #[texture(0)]
    #[sampler(1)]
    up_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    down_texture: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    left_texture: Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    right_texture: Handle<Image>,

    #[texture(8)]
    #[sampler(9)]
    forward_texture: Handle<Image>,

    #[texture(10)]
    #[sampler(11)]
    backward_texture: Handle<Image>,

    #[uniform(12)]
    skybox_intensity: f32,

    #[uniform(13)]
    fov: f32,

    #[uniform(14)]
    cam_pos: Vec3,
    #[uniform(15)]
    cam_x: Vec3, // cam right
    #[uniform(16)]
    cam_y: Vec3, // cam up
    #[uniform(17)]
    cam_z: Vec3, // the way the camera is facing

    #[texture(18)]
    #[sampler(19)]
    accretion_disc_texture: Handle<Image>,
    #[uniform(20)]
    accretion_disc_r: f32,
    #[uniform(21)]
    accretion_disc_width: f32,
    #[uniform(22)]
    accretion_disc_intensity: f32,

    #[uniform(23)]
    time: f32,
}

impl Material2d for SchwarzschildMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/schwarzschild.wgsl".into()
    }
}

fn update_material(
    cam_data: ResMut<CamData>,
    mut materials: ResMut<Assets<SchwarzschildMaterial>>,
    spacetime_params: Res<SpacetimeParams>,
    time: Res<Time>,
) {
    let mat_id = materials.ids().next().expect("Failed to get material id.");
    let mat = materials.get_mut(mat_id).expect("Failed to get material.");
    mat.cam_pos = cam_data.cam_pos;
    mat.cam_x = cam_data.cam_x;
    mat.cam_y = cam_data.cam_y;
    mat.cam_z = cam_data.cam_z;
    mat.time = time_to_geo(time.elapsed_seconds(), spacetime_params.mass) as f32;
}
/* #endregion */

// returns x, y, z
fn get_cam_axis(pos: Vec3, target: Vec3) -> (Vec3, Vec3, Vec3) {
    let cam_z = (target - pos).normalize_or_zero();
    
    let xz = vec3(cam_z.x, 0., cam_z.z).normalize_or_zero();
    let cam_x = Quat::from_axis_angle(Vec3::Y, - PI / 2.).mul_vec3(xz);

    let cam_y = Quat::from_axis_angle(cam_x, PI / 2.).mul_vec3(cam_z);

    (cam_x, cam_y, cam_z)
}

// schwarzschild
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SchwarzschildMaterial>>,
    assets: Res<AssetServer>,
    time: Res<Time>,
    spacetime_params: Res<SpacetimeParams>
) {
    let font: Handle<Font> = assets.load(FONT_PATH);

    let cam_pos = vec3(0., 10., 40.);
    let cam_target = Vec3::ZERO;

    let (cam_x, cam_y, cam_z) = get_cam_axis(cam_pos, cam_target);

    /* #region camera */
    commands
        .spawn(
            Camera3dBundle::default(),
        )
        .insert(UnrealCameraBundle::new(
            UnrealCameraController::default(),
            cam_pos,
            cam_target,
            Vec3::Y,
        ));
    /* #endregion */

    /* #region ray tracing */
    commands
        .spawn(
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::new(1., 1.)).into(),
                transform: Transform {
                    translation: Vec3::ZERO,
                    ..default()
                },
                material: materials.add(SchwarzschildMaterial {
                    up_texture: assets.load("images/skybox/skybox1/up.png"),
                    down_texture: assets.load("images/skybox/skybox1/down.png"),
                    left_texture: assets.load("images/skybox/skybox1/left.png"),
                    right_texture: assets.load("images/skybox/skybox1/right.png"),
                    forward_texture: assets.load("images/skybox/skybox1/forward.png"),
                    backward_texture: assets.load("images/skybox/skybox1/backward.png"),

                    skybox_intensity: 0.7,

                    fov: PI / 2.,

                    cam_pos,
                    cam_x,
                    cam_y,
                    cam_z,

                    accretion_disc_texture: assets.load("images/accretion-disc/disc1.png"),
                    accretion_disc_r: 6.,
                    accretion_disc_width: 12.,
                    accretion_disc_intensity: 0.8,

                    time: time_to_geo(time.elapsed_seconds(), spacetime_params.mass) as f32,
                }),
                ..default()
            }
        );

    let mut camera = Camera2dBundle::default();
    camera.camera.order = 999;
    camera.projection.scaling_mode = ScalingMode::AutoMax {
        max_width: 1.,
        max_height: 1.
    };

    commands.spawn(
        camera
    );
    /* #endregion */

    /* #region ui */
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(0.),
                bottom: Val::Px(0.),
                display: Display::Grid,
                row_gap: Val::Px(4.),
                column_gap: Val::Px(4.),
                grid_template_columns: vec![GridTrack::auto(), GridTrack::min_content()],
                grid_auto_rows: vec![GridTrack::min_content()],
                align_content: AlignContent::Start,
                padding: UiRect::all(Val::Px(8.)),
                ..default()
            },
            background_color: SIDEBAR_BG_COLOR.into(),
            ..default()
        })
        .with_children(|builder| {
            /* #region spacetime parameters */
            builder.spawn(TextBundle::from_section(
                "Spacetime Parameters",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.,
                    ..default()
                }
            ).with_style(Style {
                margin: UiRect::bottom(Val::Px(8.)),
                grid_column: GridPlacement::span(2),
                ..default()
            }));

            builder.spawn(TextBundle::from_section(
                "M (kg): ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value(spacetime_params.mass.to_string())
                    .with_inactive(true),
                Name::new("SpacetimeParamsM")
            ));
            /* #endregion */

            /* #region cosmetic */
            builder.spawn(TextBundle::from_section(
                "Cosmetics",
                TextStyle {
                    font: font.clone(),
                    font_size: 20.,
                    ..default()
                }
            ).with_style(Style {
                margin: UiRect { 
                    left: Val::Px(0.),
                    right: Val::Px(0.),
                    top: Val::Px(4.),
                    bottom: Val::Px(8.)
                },
                grid_column: GridPlacement::span(2),
                ..default()
            }));

            builder.spawn(TextBundle::from_section(
                "Up skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Down skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Left skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Right skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Forward skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Backward skybox: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));

            builder.spawn(TextBundle::from_section(
                "Accretion disc: ",
                TextStyle {
                    font: font.clone(),
                    font_size: 16.,
                    ..default()
                }
            ));
            builder.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(200.0),
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    border_color: INPUT_BORDER_COLOR_INACTIVE.into(),
                    background_color: INPUT_BG_COLOR.into(),
                    ..default()
                },
                TextInputBundle::default()
                    .with_text_style(TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: TEXT_COLOR,
                        ..default()
                    })
                    .with_value("TODO")
                    .with_inactive(true),
            ));
            /* #endregion */
        });

    /* #region position text */
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                TextBundle::from_section(
                    "",
                    TextStyle {
                        font: font.clone(),
                        font_size: 16.,
                        color: Color::WHITE,
                        ..default()
                    }
                ),
                PositionText
            ));
        });
    /* #endregion */
    /* #endregion */
}

fn focus(
    query: Query<(Entity, &Interaction), Changed<Interaction>>,
    mut text_input_query: Query<(Entity, &mut TextInputInactive, &mut BorderColor)>,
) {
    for (interaction_entity, interaction) in &query {
        if *interaction == Interaction::Pressed {
            for (entity, mut inactive, mut border_color) in &mut text_input_query {
                if entity == interaction_entity {
                    inactive.0 = false;
                    *border_color = INPUT_BORDER_COLOR_ACTIVE.into();
                } else {
                    inactive.0 = true;
                    *border_color = INPUT_BORDER_COLOR_INACTIVE.into();
                }
            }
        }
    }
}

/* #region window data */
#[derive(Resource, Default)]
struct WindowData {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn update_window_data(
    mut window_data: ResMut<WindowData>,
    window: Query<&Window>
) {
    let window = window.single();

    let width = window.width();
    let height = window.height();

    let (x, y) = match window.position {
        WindowPosition::At(v) => (v.x as f32, v.y as f32),
        _ => (0., 0.),
    };

    window_data.x = x as u32;
    window_data.y = y as u32;
    window_data.width = width as u32;
    window_data.height = height as u32;
}
/* #endregion */

/* #region camera data */
#[derive(Resource, Default, Debug)]
struct CamData {
    cam_pos: Vec3,
    cam_x: Vec3,
    cam_y: Vec3,
    cam_z: Vec3,
}

fn update_camera_data(
    mut cam: Query<&LookTransform>,
    mut cam_data: ResMut<CamData>
) {
    if let Ok(transform) = cam.get_single_mut() {
        let (cam_x, cam_y, cam_z) = get_cam_axis(transform.eye, transform.target);

        cam_data.cam_pos = transform.eye;
        cam_data.cam_x = cam_x;
        cam_data.cam_y = cam_y;
        cam_data.cam_z = cam_z;
    }
}
/* #endregion */

/* #region text with position */
#[derive(Component)]
struct PositionText;

fn update_position_text(
    mut query: Query<&mut Text, With<PositionText>>,
    cam_data: Res<CamData>,
    spacetime_params: Res<SpacetimeParams>
) {
    let mut text = query.get_single_mut().expect("Failed to get text with PositionText.");

    let rs = length_to_si(2., spacetime_params.mass);

    let r = cam_data.cam_pos.length();
    let delta_r = length_to_si(r - 2., spacetime_params.mass);

    let proper_length = length_to_si(r.sqrt() * (r - 2.).sqrt() + f32::ln(r + r.sqrt() * (r - 2.).sqrt() - 1.), spacetime_params.mass);

    text.sections[0].value = format!("Schwarzschild radius: {rs}\nDifference in r from event horizon: {delta_r} m\nProper distance from event horizon: {proper_length} m");
}
/* #endregion */

/* #region spacetime parameters */
#[derive(Resource)]
struct SpacetimeParams {
    mass: f64
}

impl Default for SpacetimeParams {
    fn default() -> Self {
        SpacetimeParams {
            mass: 1e34 as f64
        }
    }
}

fn update_spacetime_params(
    query: Query<(&TextInputValue, &Name)>,
    mut spacetime_params: ResMut<SpacetimeParams>
) {
    for (text_input, name) in &query {
        if name.contains("SpacetimeParamsM") {
            let value: f64 = text_input.0.parse().unwrap_or(0.);
            spacetime_params.mass = value;
        }
    }
}
/* #endregion */
