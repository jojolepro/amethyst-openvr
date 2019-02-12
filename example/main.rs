extern crate amethyst;
extern crate amethyst_gltf;
extern crate amethyst_openvr;
#[macro_use]
extern crate serde;
//extern crate amethyst_xr_models;

mod tracker_system;

use amethyst::utils::scene::BasicScenePrefab;
use amethyst::assets::{AssetPrefab, PrefabLoader, RonFormat, PrefabLoaderSystem, PrefabData, ProgressCounter};
use amethyst::animation::{AnimationBundle, VertexSkinningBundle};
use amethyst_gltf::{GltfSceneAsset, GltfSceneFormat, GltfSceneLoaderSystem};
use amethyst::core::nalgebra::{Matrix4, Vector3};
use amethyst::core::transform::{GlobalTransform, Transform, TransformBundle};
use amethyst::core::specs::{Entity, Join};
use amethyst::input::{is_close_requested, is_key_down, InputBundle};
use amethyst::prelude::*;
use amethyst::derive::*;
use amethyst::renderer::{
    ActiveCamera, Camera, LightPrefab, DrawPbmSeparate, Light, PointLight, PosNormTangTex, Projection, DepthMode, ALPHA, ColorMask, VirtualKeyCode,DisplayConfig, Pipeline, Stage, DrawShaded, DrawSkybox, PosNormTex, RenderBundle
};
use amethyst::ui::UiBundle;
use amethyst::utils::fps_counter::FPSCounterBundle;
use amethyst::Error;

use serde::{Deserialize, Serialize};

use amethyst::xr::{XRBundle, XREvent};
use amethyst_openvr::{ApplicationType, OpenVR};

//use amethyst_xr_models::{XRTrackerModels};


#[derive(Default, Deserialize, Serialize, PrefabData)]
#[serde(default)]
struct MyPrefabData {
    transform: Option<Transform>,
    gltf: Option<AssetPrefab<GltfSceneAsset, GltfSceneFormat>>,
    light: Option<LightPrefab>,
}

#[derive(Default)]
struct VRExample;

impl SimpleState for VRExample {
    fn on_start(&mut self, data: StateData<GameData>) {
        let prefab_handle = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load("prefab.ron", RonFormat, (), ())
        });

        data.world.create_entity().with(prefab_handle).with(Transform::from(Vector3::new(-5.0, 0.0, 10.0))).build();

        let cam = data.world
            .create_entity()
            .with(Transform::default())
            //.with(Camera::from(Projection::perspective(1.3, f32::consts::FRAC_PI_6)))
            .with(Camera::standard_3d(1920.0, 1080.0))
            .build();

        data.world.add_resource(ActiveCamera { entity: Some(cam) });

        let light1: Light = PointLight {
            intensity: 1.0,
            color: [0.9, 0.9, 0.9].into(),
            ..PointLight::default()
        }.into();

        data.world
            .create_entity()
            .with(light1)
            .with(Transform::from(Vector3::new(0.0, 10.0, 0.0)))
            .build();
    }

    fn handle_event(
        &mut self,
        _: StateData<GameData>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }

        Trans::None
    }

    fn update(&mut self, data: &mut StateData<GameData>) -> SimpleTrans {
        data.data.update(&data.world);

        //(&data.world.read_storage::<Transform>(), &data.world.read_storage::<Camera>()).join().for_each(|(tr, _)| println!("Cam transform: {:?}", tr));

        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let resources_directory = format!("{}/example/resources/", env!("CARGO_MANIFEST_DIR"));

    let display_config_path = format!(
        "{}/example/resources/display_config.ron",
        env!("CARGO_MANIFEST_DIR")
    );

    let mut game_data = GameDataBuilder::default();

    if OpenVR::is_available() {
        let openvr = OpenVR::init(ApplicationType::Scene)?;
        game_data = game_data.with_bundle(XRBundle::new(openvr))?;
    }

    let render_bundle = {
        let display_config = DisplayConfig::load(&display_config_path);
        let pipe = Pipeline::build().with_stage(
            Stage::with_backbuffer()
                .clear_target([0.0, 0.0, 0.2, 1.0], 1.0)
                .with_pass(DrawPbmSeparate::new().with_vertex_skinning().with_transparency(ColorMask::all(), ALPHA, Some(DepthMode::LessEqualWrite)))
                .with_pass(DrawSkybox::new()),
        );
        RenderBundle::new(pipe, Some(display_config))
    };

    game_data = game_data
        .with(PrefabLoaderSystem::<MyPrefabData>::default(), "scene_loader", &[])
        .with(
            GltfSceneLoaderSystem::default(),
            "gltf_loader",
            &["scene_loader"], // This is important so that entity instantiation is performed in a single frame.
        )
        .with(
            tracker_system::TrackerSystem::default(),
            "tracker_system",
            &[],
        )
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<String, String>::new())?
        .with_bundle(FPSCounterBundle::default())?
        .with_bundle(
            AnimationBundle::<usize, Transform>::new("animation_control", "sampler_interpolation"),
        )?
        .with_bundle(VertexSkinningBundle::new())?
        //.with_basic_renderer(display_config_path, DrawPbm::<PosNormTangTex>::new(), true)?
        .with_bundle(render_bundle)?
        .with_bundle(InputBundle::<String, String>::new())?;
        //.with(XRTrackerModels, "tracker_models", &[]);

    let mut game = Application::build(resources_directory, VRExample::default())?
        .register::<amethyst::core::Named>()
        .build(game_data)?;
    game.run();

    Ok(())
}
