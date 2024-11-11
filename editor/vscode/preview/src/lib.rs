use std::{cell::RefCell, rc::Rc};

use event::Message;
use nalgebra::{point, vector, Matrix4};
use wasm_bindgen::prelude::*;

mod event;
mod json;
mod model;
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

  let current = Rc::new(RefCell::new(None));

  {
    let current = current.clone();
    event::listen(move |m: Message| match m {
      Message::RenderModel { model } => {
        info!("rendering model {:?}", model);

        current.borrow_mut().take();

        let buffers = model::render(&model);
        let render = Render::new(buffers).unwrap();

        let preview = Preview::new();
        render.set_matrices(&preview.proj.data.as_slice(), &preview.view.data.as_slice());
        let preview = Rc::new(RefCell::new(preview));

        let textures = model.textures.clone();
        let texture_names = textures.values().cloned().collect();

        let current = current.clone();
        render.context.clone().load_images(&texture_names, move |textures| {
          for t in textures.values() {
            t.load(&render.context);
          }

          let preview_2 = preview.clone();
          let handle = render.setup_loop(move |render| {
            render.clear();
            preview_2.borrow_mut().update();

            preview_2.borrow_mut().draw(render);
          });

          *current.borrow_mut() = Some((preview, handle));
        });
      }
    });
  }

  {
    let current = current.clone();
    event::on_mouse_move(move |x, y| {
      if let Some((preview, _)) = current.borrow().as_ref() {
        preview.borrow_mut().mouse_move(x, y);
      }
    });
  }

  {
    let current = current.clone();
    event::on_mouse_down(move || {
      if let Some((preview, _)) = current.borrow().as_ref() {
        preview.borrow_mut().mouse_down();
      }
    });
  }
  {
    let current = current.clone();
    event::on_mouse_up(move || {
      if let Some((preview, _)) = current.borrow().as_ref() {
        preview.borrow_mut().mouse_up();
      }
    });
  }

  Ok(())
}

impl Preview {
  fn new() -> Self {
    Preview {
      proj:  Matrix4::new_perspective(1.0, 1.0, 0.1, 100.0),
      view:  Matrix4::look_at_rh(
        &point![0.0, 1.5, 2.0],
        &point![0.0, 0.0, 0.0],
        &vector![0.0, 1.0, 0.0],
      ),
      model: Matrix4::identity(),

      rotation_yaw: 0.0,
    }
  }

  pub fn mouse_move(&mut self, _x: f32, _y: f32) {}
  pub fn mouse_down(&mut self) {}
  pub fn mouse_up(&mut self) {}

  fn update(&mut self) {
    self.rotation_yaw += 0.01;

    self.model = Matrix4::new_rotation(&vector![0.0, 1.0, 0.0] * self.rotation_yaw as f32)
      * Matrix4::new_translation(&vector![-0.5, -0.5, -0.5]);
  }

  fn draw(&mut self, render: &Render) { render.draw(self.model.data.as_slice()); }
}
