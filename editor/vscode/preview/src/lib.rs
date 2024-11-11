use event::Message;
use nalgebra::{point, vector, Matrix4, Vector3};
use wasm_bindgen::prelude::*;

mod event;
mod render;

use render::Render;

#[macro_use]
extern crate log;

struct Preview {
  proj:  Matrix4<f32>,
  view:  Matrix4<f32>,
  model: Matrix4<f32>,

  rotation_yaw: f64,
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
  console_log::init().unwrap();

  event::listen(|m: Message| match m {
    Message::RenderModel { model } => {
      info!("rendering model {:?}", model);

      let render = Render::new().unwrap();

      let mut preview = Preview::new();
      render.set_matrices(&preview.proj.data.as_slice(), &preview.view.data.as_slice());

      let textures = model.textures.clone();
      let texture_names = textures.values().cloned().collect();

      render.clone().load_images(&texture_names, move |textures| {
        for t in textures.values() {
          t.load(&render.context);
        }

        render.setup_loop(move |render| {
          render.clear();
          preview.update();

          for element in &model.elements {
            let min =
              Vector3::new(element.from[0] / 16.0, element.from[1] / 16.0, element.from[2] / 16.0);
            let max =
              Vector3::new(element.to[0] / 16.0, element.to[1] / 16.0, element.to[2] / 16.0);

            let scale = max - min;
            let translation = min;
            let transform =
              Matrix4::new_nonuniform_scaling(&scale) * Matrix4::new_translation(&translation);

            preview.draw(render, transform);
          }
        });
      });
    }
  });

  Ok(())
}

impl Preview {
  fn new() -> Self {
    Preview {
      proj:  Matrix4::new_perspective(1.0, 1.0, 0.1, 100.0),
      view:  Matrix4::look_at_rh(
        &point![0.0, 4.0, 5.0],
        &point![0.0, 0.0, 0.0],
        &vector![0.0, 1.0, 0.0],
      ),
      model: Matrix4::identity(),

      rotation_yaw: 0.0,
    }
  }

  fn update(&mut self) {
    self.rotation_yaw += 0.01;

    self.model = Matrix4::new_rotation(&vector![0.0, 1.0, 0.0] * self.rotation_yaw as f32)
      * Matrix4::new_translation(&vector![-0.5, -0.5, -0.5]);
  }

  fn draw(&mut self, render: &Render, transform: Matrix4<f32>) {
    let model = self.model * transform;
    render.draw(model.data.as_slice());
  }
}
