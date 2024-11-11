use nalgebra::{point, vector, Matrix4};
use wasm_bindgen::prelude::*;

mod render;

use render::Render;

struct Preview {
  proj:  Matrix4<f32>,
  view:  Matrix4<f32>,
  model: Matrix4<f32>,

  rotation_yaw: f64,
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
  let render = Render::new()?;

  let mut preview = Preview::new();

  let texture = "";
  render.clone().load_images(&[texture], |textures| {
    textures[texture].bind(&render);

    render.setup_loop(move |render| {
      preview.draw(render);
    });
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

  fn draw(&mut self, render: &Render) {
    self.rotation_yaw += 0.01;

    self.model = Matrix4::new_rotation(&vector![0.0, 1.0, 0.0] * self.rotation_yaw as f32)
      * Matrix4::new_translation(&vector![-0.5, -0.5, -0.5]);

    render.draw(
      &self.proj.data.as_slice(),
      &self.view.data.as_slice(),
      &self.model.data.as_slice(),
    );
  }
}
