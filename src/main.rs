use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    render::{camera::ScalingMode, render_resource::encase::vector::FromVectorParts},
    ui::update,
};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};

mod hilbert;
mod moore;
use hilbert::Hilbert;
use moore::Moore;

#[derive(Component)]
struct Traveller {
    position: f32,
    destination: f32,
    speed: f32,
    acceleration: f32,
}

#[derive(Component)]
struct MainShape {}

#[derive(Component)]
struct TravelDot {}

#[derive(Component)]
struct Mover {
    t_last: f32,
    pos: f32,
    direction: f32,
    it: f32,
}

#[derive(Default, Resource)]
struct MeshHandles {
    ring: Handle<Mesh>,
    dot: Handle<Mesh>,
    base_col: Handle<ColorMaterial>,
    bg_col: Handle<ColorMaterial>,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(Srgba::hex("002B36FF").unwrap())))
        .init_resource::<MeshHandles>()
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Update, movers)
        .add_systems(Update, close_on_esc)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, draw_lines)
        .add_systems(
            Update,
            (simple_move, update_traveller, update_sizes, update_sizes2),
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut config_store: ResMut<GizmoConfigStore>,
    mut mesh_handles: ResMut<MeshHandles>,
) {
    // Camera
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1.05,
                min_height: 1.05,
            },
            ..OrthographicProjection::default_2d()
        },
    ));

    let (gizmo_cfg, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmo_cfg.depth_bias = 1.0;

    mesh_handles.ring = meshes.add(Annulus::new(0.8, 1.0));
    mesh_handles.dot = meshes.add(Circle::new(1.0));
    mesh_handles.base_col = materials.add(Color::WHITE);
    mesh_handles.bg_col = materials.add(Color::Srgba(Srgba::hex("002B36FF").unwrap()));

    // Hilbert Curve
    let moore = Moore::new(6);
    let mut rng = thread_rng();
    let dist = Normal::new(0., 0.2).unwrap();
    for i in (0..(moore.total_size) as isize).step_by(2) {
        let extent = moore.dim_size as f32;
        let pos = moore.forward_circular(i);
        //        let i = rng.gen_range(0..moore.total_size) as isize;
        //        let pos = moore.forward_circular(i);
        let pos = Vec3::new(pos.0 as f32, pos.1 as f32, 0.) / extent - 0.5;
        commands
            .spawn((
                Traveller {
                    position: i as f32,
                    acceleration: 0.5,
                    destination: i as f32,
                    speed: 2.0 + dist.sample(&mut rng),
                },
                Transform {
                    translation: pos,
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent
                    .spawn((MainShape {}, Transform::IDENTITY))
                    .with_children(|ms| {
                        ms.spawn((
                            Mesh2d(mesh_handles.ring.clone()),
                            MeshMaterial2d(mesh_handles.base_col.clone()),
                            Transform::from_scale(Vec3::splat(0.4 / extent)),
                        ));
                        ms.spawn((
                            Mesh2d(mesh_handles.dot.clone()),
                            MeshMaterial2d(mesh_handles.bg_col.clone()),
                            Transform::from_scale(Vec3::splat(0.4 * 0.8 / extent))
                                .with_translation(Vec3::new(0., 0., 0.5)),
                        ));
                        if rng.gen_bool(0.5) {
                            ms.spawn((
                                Mesh2d(mesh_handles.dot.clone()),
                                MeshMaterial2d(mesh_handles.base_col.clone()),
                                Transform::from_scale(Vec3::splat(0.1 / extent))
                                    .with_translation(Vec3::new(0., 0., 1.)),
                            ));
                        }
                    });
                parent
                    .spawn((
                        TravelDot {},
                        Transform::IDENTITY
                            .with_scale(Vec3::splat(0.0))
                            .with_translation(Vec3::new(0.0, 0.0, 2.0)),
                    ))
                    .with_children(|td| {
                        td.spawn((
                            Mesh2d(mesh_handles.dot.clone()),
                            MeshMaterial2d(mesh_handles.base_col.clone()),
                            Transform::from_scale(Vec3::splat(0.2 / extent)),
                        ));
                    });
            });
    }
    let dist = 400.;
    let mut cur = dist / 2.;
    while cur < moore.total_size as f32 {
        commands.spawn(Mover {
            t_last: 0.,
            pos: cur,
            direction: -8.,
            it: 0.05,
        });
        cur += dist;
    }
    commands.spawn(moore);
}

fn draw_lines(mut gizmos: Gizmos, hilberts: Query<&Moore>) {
    let hilbert = hilberts.single();
    let extent = hilbert.dim_size as f32;
    for pos in (0..hilbert.total_size) {
        let from = hilbert.forward[pos];
        let to = hilbert.forward[(pos + 1) % hilbert.total_size];
        let from = Vec2::new(from.0 as f32, from.1 as f32) / extent - 0.5;
        let to = Vec2::new(to.0 as f32, to.1 as f32) / extent - 0.5;

        gizmos.line_2d(from, to, Color::srgb(0.4, 0.4, 0.5));
    }
}

fn simple_move(mut trvs: Query<&mut Traveller>, time: Res<Time<Real>>) {
    for mut trv in trvs.iter_mut() {
        let s = Vec2::new(trv.position, 0.);
        let d = Vec2::new(trv.destination, 0.);
        let dt = time.delta_secs();
        let dist = trv.speed * dt;
        trv.position = s.move_towards(d, dist).x;
    }
}

fn update_traveller(mut trvs: Query<(&mut Traveller, &mut Transform)>, sfc: Query<&Moore>) {
    let sfc = sfc.single();
    let extent = sfc.dim_size as f32;
    for (trv, mut transform) in trvs.iter_mut() {
        let (from, to, ratio) = (
            sfc.forward_circular(trv.position.floor() as isize),
            sfc.forward_circular((trv.position.floor() + 1.1).floor() as isize),
            trv.position.fract(),
        );
        let from = Vec2::new(from.0 as f32, from.1 as f32) / extent - 0.5;
        let to = Vec2::new(to.0 as f32, to.1 as f32) / extent - 0.5;
        let pos = (to - from) * ratio + from;
        *transform = transform.with_translation(Vec3::new(pos.x, pos.y, 0.));
    }
}

fn update_sizes(
    trvs: Query<&Traveller>,
    mut mainshapes: Query<(&Parent, &mut Transform), With<MainShape>>,
    time: Res<Time<Real>>,
) {
    let dt = time.delta_secs();
    for (p, mut ms) in mainshapes.iter_mut() {
        if let Ok(mut trv) = trvs.get(p.get()) {
            if (trv.destination - trv.position).abs() > 1.0 {
                *ms = ms.with_scale(ms.scale.move_towards(Vec3::ZERO, 2.0 * dt));
            } else {
                *ms = ms.with_scale(ms.scale.move_towards(Vec3::ONE, 2.0 * dt))
            }
        }
    }
}

fn update_sizes2(
    trvs: Query<&Traveller>,
    mut mainshapes: Query<(&Parent, &mut Transform), With<TravelDot>>,
    time: Res<Time<Real>>,
) {
    let dt = time.delta_secs();
    for (p, mut ms) in mainshapes.iter_mut() {
        if let Ok(mut trv) = trvs.get(p.get()) {
            if (trv.destination - trv.position).abs() > 0.4 {
                *ms = ms.with_scale(ms.scale.move_towards(Vec3::ONE, 4.0 * dt));
            } else {
                *ms = ms.with_scale(ms.scale.move_towards(Vec3::ZERO, 2.0 * dt))
            }
        }
    }
}

fn random_move(mut trvs: Query<&mut Traveller>) {
    let mut rng = thread_rng();
    for mut trv in trvs.iter_mut() {
        if rng.gen_bool(0.01) {
            trv.destination += 2.;
        }
    }
}

fn movers(
    mut trvs: Query<&mut Traveller>,
    mut movers: Query<&mut Mover>,
    sfc: Query<&Moore>,
    time: Res<Time<Real>>,
) {
    let sfc = sfc.single();
    let l = sfc.total_size as f32;
    let mut rng = thread_rng();
    for mut mvr in movers.iter_mut() {
        if mvr.t_last + mvr.it < time.elapsed_secs() {
            mvr.t_last = time.elapsed_secs();
            for mut trv in trvs.iter_mut() {
                let trv_pos = ((trv.position % l) + l) % l;
                let mvr_pos = ((mvr.pos % l) + l) % l;
                if ((mvr_pos <= trv_pos) && (trv_pos <= mvr_pos + mvr.direction))
                    || ((mvr_pos + mvr.direction <= trv_pos) && (trv_pos <= mvr_pos))
                {
                    trv.destination += 4.;
                }
            }
            mvr.pos += mvr.direction;
        }
    }
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
