use three_d::*;

/* reference examples

https://github.com/asny/three-d/tree/master/examples/wireframe
*/

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "3d model viewer".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl();
    let viewport = window.viewport();
    let dpi = window.device_pixel_ratio();

    let target = vec3(0., 0., 0.); // make adjustable later
    let scene_rad: f32 = 6.0; // NEEDS to be defined as f32
    let mut camera = Camera::new_perspective(
        viewport,
        target + scene_rad * vec3(0.6, 0.3, 1.0).normalize(),
        target,
        vec3(0., 1., 0.),
        degrees(60.),
        0.1,
        1000.,
    );
    let mut controller = OrbitControl::new(camera.target(), 0.1 * scene_rad, 100. * scene_rad);

    let mut loaded =
        if let Ok(loaded) = three_d_asset::io::load_async(&["../assets/b21vdWIPRsa.stl"]).await {
            loaded
        } else {
            panic!("Failed to load model file");
        };

    let cpu_mesh = loaded.deserialize("b21vdWIPRsa").unwrap();
    let model = Gm::new(
        Mesh::new(&context, &cpu_mesh),
        PhysicalMaterial::new(
            &context,
            &CpuMaterial {
                albedo: Srgba::RED,
                ..Default::default()
            },
        ),
    );

    // default light, add options for custom lights later
    let light = AmbientLight::new(&context, 1.0, Srgba::WHITE);

    let mut gui = three_d::GUI::new(&context);

    let clear_color = [0.8; 4];
    window.render_loop(move |mut frame_input| {
        let mut panel_w = 0.;
        gui.update(
            &mut frame_input.events,
            frame_input.accumulated_time,
            frame_input.viewport,
            frame_input.device_pixel_ratio,
            |g_c| {
                use three_d::egui::*;

                SidePanel::left("side_panel").show(g_c, |ui| {
                    ui.set_min_size(vec2(300., 1000.));
                    ui.heading("Configuration Panel");

                    /*if ui.button("Select a model").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("3D Models", &["stl", "obj"])
                            .pick_file()
                        {
                            println!("selected file: {:?}", path);
                        }
                    }*/
                });

                panel_w = g_c.used_rect().width();
            },
        );

        let viewport = Viewport {
            x: (panel_w * frame_input.device_pixel_ratio) as i32,
            y: 0,
            width: frame_input.viewport.width - (panel_w * frame_input.device_pixel_ratio) as u32,
            height: frame_input.viewport.height,
        };
        camera.set_viewport(viewport);
        controller.handle_events(&mut camera, &mut frame_input.events);

        let screen = frame_input.screen();
        screen.clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0));

        screen.render(&camera, &[&model], &[&light]);

        screen.write(|| gui.render()).unwrap();
        FrameOutput::default()
    });
}
