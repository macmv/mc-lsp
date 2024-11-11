use core::f32;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use event::Message;
use nalgebra::{point, vector, Matrix4, UnitQuaternion, Vector3};
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

  rotation_yaw:   f32,
  rotation_pitch: f32,
  zoom:           f32,

  mouse_down: bool,
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
        let preview = Rc::new(RefCell::new(preview));

        let textures = model.textures.clone();
        let texture_names = textures.values().cloned().collect();

        let current = current.clone();
        render.context.clone().load_images(&texture_names, move |textures| {
          let width = textures.values().map(|t| t.width()).sum();
          let height = textures.values().map(|t| t.height()).max().unwrap();
          render.context.setup_image(width, height);

          let mut uv_map = HashMap::new();

          let mut x = 0;
          for (k, t) in textures {
            uv_map.insert(k, (x as f32 / width as f32, t.width() as f32 / width as f32));

            t.load(&render.context, x, 0);
            x += t.width() as i32;
          }

          let preview_2 = preview.clone();
          let handle = render.setup_loop(move |render| {
            render.clear();
            preview_2.borrow_mut().update();

            preview_2.borrow().draw(render);
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

      rotation_pitch: -f32::consts::PI / 6.0,
      rotation_yaw:   f32::consts::PI / 4.0,
      zoom:           2.0,

      mouse_down: false,
    }
  }

  pub fn mouse_move(&mut self, x: f32, y: f32) {
    if self.mouse_down {
      self.rotation_pitch -= y * 0.005;
      self.rotation_yaw -= x * 0.005;

      const MAX_PITCH: f32 = f32::consts::PI / 2.0 - 0.01;

      self.rotation_pitch = self.rotation_pitch.clamp(-MAX_PITCH, MAX_PITCH);
    }
  }
  pub fn mouse_down(&mut self) { self.mouse_down = true; }
  pub fn mouse_up(&mut self) { self.mouse_down = false; }

  fn update(&mut self) {
    let y_axis = Vector3::y_axis();
    let x_axis = Vector3::x_axis();

    let rotation = UnitQuaternion::from_axis_angle(&y_axis, self.rotation_yaw)
      * UnitQuaternion::from_axis_angle(&x_axis, self.rotation_pitch);

    self.view = Matrix4::look_at_rh(
      &(rotation * point![0.0, 0.0, self.zoom]),
      &point![0.0, 0.0, 0.0],
      &Vector3::y_axis(),
    );
    self.model = Matrix4::new_translation(&vector![-0.5, -0.5, -0.5]);
  }

  fn draw(&self, render: &Render) {
    render.set_matrices(
      self.proj.data.as_slice(),
      self.view.data.as_slice(),
      self.model.data.as_slice(),
    );
    render.draw();
  }
}
