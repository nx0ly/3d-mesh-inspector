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
    let mut example_mat = PhysicalMaterial::new_opaque(
        &context,
        &CpuMaterial {
            albedo: Srgba::new_opaque(255, 255, 255),
            roughness: 0.7,
            metallic: 0.8,
            ..Default::default()
        },
    );
    example_mat.render_states.cull = Cull::Back;
    let example_model = Gm::new(Mesh::new(&context, &cpu_mesh), example_mat);

    // wireframe schenanigans
    let mut wireframe_mat = PhysicalMaterial::new(
        &context,
        &CpuMaterial {
            albedo: Srgba::new_opaque(255, 0, 0),
            metallic: 0.7,
            roughness: 0.8,
            ..Default::default()
        },
    );
    wireframe_mat.render_states.cull = Cull::Back;

    // we will use cylinders to draw out the dividing lines
    // in the three_d wireframe example it also uses circles to display vertices
    // idrc i dont want to do that
    let mut cylinder = CpuMesh::cylinder(4);
    cylinder
        .transform(Mat4::from_nonuniform_scale(1.0, 0.007, 0.007))
        .unwrap();
    let edges = Gm::new(
        InstancedMesh::new(&context, &edge_transformations(&cpu_mesh), &cylinder),
        wireframe_mat.clone(),
    );

    // default light, add options for custom lights later
    let ambient_light = AmbientLight::new(&context, 1.0, Srgba::WHITE);

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

        screen.render(
            &camera,
            example_model.into_iter().chain(&edges),
            &[&ambient_light],
        );

        screen.write(|| gui.render()).unwrap();
        FrameOutput::default()
    });
}

fn edge_transformations(cpu_mesh: &CpuMesh) -> Instances {
    let indices = cpu_mesh.indices.to_u32().unwrap();
    let positions = cpu_mesh.positions.to_f32();
    let mut transformations = Vec::new();
    for f in 0..indices.len() / 3 {
        let i1 = indices[3 * f] as usize;
        let i2 = indices[3 * f + 1] as usize;
        let i3 = indices[3 * f + 2] as usize;

        if i1 < i2 {
            transformations.push(edge_transform(positions[i1], positions[i2]));
        }
        if i2 < i3 {
            transformations.push(edge_transform(positions[i2], positions[i3]));
        }
        if i3 < i1 {
            transformations.push(edge_transform(positions[i3], positions[i1]));
        }
    }
    Instances {
        transformations,
        ..Default::default()
    }
}

fn edge_transform(p1: Vec3, p2: Vec3) -> Mat4 {
    Mat4::from_translation(p1)
        * Into::<Mat4>::into(Quat::from_arc(
            vec3(1.0, 0.0, 0.0),
            (p2 - p1).normalize(),
            None,
        ))
        * Mat4::from_nonuniform_scale((p1 - p2).magnitude(), 1.0, 1.0)
}
