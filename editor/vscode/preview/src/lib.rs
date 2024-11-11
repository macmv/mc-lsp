use nalgebra::Matrix4;
use wasm_bindgen::prelude::*;

mod render;

use render::Render;

struct Preview {
  render: Render,
  proj:   Matrix4<f32>,
  view:   Matrix4<f32>,
  model:  Matrix4<f32>,
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
  let render = Render::new()?;

  let preview = Preview::new(render);

  // return Err(format!("{}", preview.proj).into());

  preview.draw();

  Ok(())
}

impl Preview {
  fn new(render: Render) -> Self {
    Preview {
      render,
      proj: Matrix4::new_perspective(1.0, 1.0, 0.1, 100.0),
      view: Matrix4::look_at_rh(
        &nalgebra::Point3::new(0.0, 4.0, 5.0),
        &nalgebra::Point3::new(0.0, 0.0, 0.0),
        &nalgebra::Vector3::new(0.0, 1.0, 0.0),
      ),
      model: Matrix4::identity(),
    }
  }

  fn draw(&self) {
    self.render.draw(
      &self.proj.data.as_slice(),
      &self.view.data.as_slice(),
      &self.model.data.as_slice(),
    );
  }
}
