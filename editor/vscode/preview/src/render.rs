use std::{
  cell::{Cell, RefCell},
  collections::{HashMap, HashSet},
  rc::Rc,
};

use wasm_bindgen::prelude::*;
use web_sys::{
  HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
  WebGlUniformLocation,
};

use WebGl2RenderingContext as gl;

use crate::model::Buffers;

pub struct Render {
  pub context: Context,
  buffers:     Buffers,

  proj_uniform_location:  Option<WebGlUniformLocation>,
  view_uniform_location:  Option<WebGlUniformLocation>,
  model_uniform_location: Option<WebGlUniformLocation>,

  tex_uniform_location: Option<WebGlUniformLocation>,
}

#[derive(Clone)]
pub struct Context {
  context: WebGl2RenderingContext,
  texture: WebGlTexture,
}

pub struct Image {
  image: Rc<RefCell<HtmlImageElement>>,
}

impl Render {
  pub fn new(context: Context, buffers: Buffers) -> Result<Self, String> {
    let vert_shader = context.compile_shader(gl::VERTEX_SHADER, include_str!("vert.glsl"))?;

    let frag_shader = context.compile_shader(gl::FRAGMENT_SHADER, include_str!("frag.glsl"))?;
    let program = context.link_program(&vert_shader, &frag_shader)?;
    context.context.use_program(Some(&program));

    let vao =
      context.context.create_vertex_array().ok_or("Could not create vertex array object")?;
    context.context.bind_vertex_array(Some(&vao));

    context.create_f32_buffer(bytemuck::cast_slice(&buffers.pos))?;
    let pos_attribute_location = context.context.get_attrib_location(&program, "pos");
    context.context.vertex_attrib_pointer_with_i32(
      pos_attribute_location as u32,
      3,
      gl::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(pos_attribute_location as u32);

    context.create_f32_buffer(bytemuck::cast_slice(&buffers.uv))?;
    let uv_attribute_location = context.context.get_attrib_location(&program, "uv");
    context.context.vertex_attrib_pointer_with_i32(
      uv_attribute_location as u32,
      2,
      gl::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(uv_attribute_location as u32);

    context.create_f32_buffer(bytemuck::cast_slice(&buffers.normal))?;
    let normal_attribute_location = context.context.get_attrib_location(&program, "normal");
    context.context.vertex_attrib_pointer_with_i32(
      normal_attribute_location as u32,
      3,
      gl::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(normal_attribute_location as u32);

    context.context.enable(gl::DEPTH_TEST);
    context.context.enable(gl::CULL_FACE);

    context.context.enable(gl::BLEND);
    context.context.blend_func(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

    Ok(Render {
      proj_uniform_location: context.context.get_uniform_location(&program, "proj"),
      view_uniform_location: context.context.get_uniform_location(&program, "view"),
      model_uniform_location: context.context.get_uniform_location(&program, "model"),
      tex_uniform_location: context.context.get_uniform_location(&program, "tex"),
      context,
      buffers,
    })
  }

  pub fn clear(&self) {
    self.context.context.clear_color(0.6, 0.7, 0.7, 1.0);
    self.context.context.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
  }

  pub fn set_matrices(&self, proj: &[f32], view: &[f32], model: &[f32]) {
    self.context.context.uniform_matrix4fv_with_f32_array(
      self.proj_uniform_location.as_ref(),
      false,
      proj,
    );
    self.context.context.uniform_matrix4fv_with_f32_array(
      self.view_uniform_location.as_ref(),
      false,
      view,
    );
    self.context.context.uniform_matrix4fv_with_f32_array(
      self.model_uniform_location.as_ref(),
      false,
      model,
    );
  }

  pub fn draw(&self) {
    self.context.context.uniform1i(self.tex_uniform_location.as_ref(), 0);

    self.context.context.draw_arrays(gl::TRIANGLES, 0, self.buffers.pos.len() as i32);
  }

  pub fn setup_loop(self, mut f: impl FnMut(&Render) + 'static) -> LoopHandle {
    // Don't look too close, you're eyes might fall out.
    let render_func: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
    let render_func_2 = render_func.clone();

    let running = Rc::new(Cell::new(true));

    let running_2 = running.clone();
    *render_func.borrow_mut() = Some(Closure::new(move || {
      f(&self);

      if running_2.get() {
        let window = web_sys::window().unwrap();
        window
          .request_animation_frame(
            render_func_2.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
          )
          .unwrap();
      }
    }));

    let window = web_sys::window().unwrap();
    window
      .request_animation_frame(render_func.borrow().as_ref().unwrap().as_ref().unchecked_ref())
      .unwrap();

    LoopHandle { running }
  }
}

#[must_use = "the loop will stop when the handle is dropped"]
pub struct LoopHandle {
  running: Rc<Cell<bool>>,
}
impl Drop for LoopHandle {
  fn drop(&mut self) { self.running.set(false); }
}

impl Context {
  pub fn new() -> Result<Self, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

    canvas.set_width(800);
    canvas.set_height(800);
    canvas.style().set_property("width", "400px")?;
    canvas.style().set_property("height", "400px")?;

    Ok(Context { texture: context.create_texture().unwrap(), context })
  }

  pub fn load_images(
    &self,
    paths: &HashSet<String>,
    on_load: impl FnOnce(&HashMap<String, Image>) + 'static,
  ) {
    if paths.is_empty() {
      on_load(&HashMap::new());
      return;
    }

    let mut images = HashMap::new();
    let done = Rc::new(RefCell::new(HashMap::new()));
    let total = paths.len();
    let on_load = Rc::new(RefCell::new(Some(on_load)));
    for path in paths {
      let image = HtmlImageElement::new().unwrap();
      image.set_src(path);

      let rc = Rc::new(RefCell::new(image));

      let image = Image { image: rc.clone() };
      images.insert(path.to_string(), image);
    }

    let images = Rc::new(images);

    // Welcome to Clone City!
    for (path, image) in images.iter() {
      let path = path.clone();
      let done = done.clone();
      let on_load = on_load.clone();
      let images = images.clone();

      let closure = Closure::wrap(Box::new(move || {
        let mut done = done.borrow_mut();
        done.insert(path.clone(), ());
        if done.len() == total {
          on_load.take().unwrap()(&images);
        }
      }) as Box<dyn FnMut()>);

      image.image.borrow_mut().set_onload(Some(closure.as_ref().unchecked_ref()));

      std::mem::forget(closure);
    }
  }

  pub fn create_f32_buffer(&self, data: &[f32]) -> Result<WebGlBuffer, String> {
    let buffer = self.context.create_buffer().ok_or("Failed to create buffer")?;
    self.context.bind_buffer(gl::ARRAY_BUFFER, Some(&buffer));
    unsafe {
      let positions_array_buf_view = js_sys::Float32Array::view(data);

      self.context.buffer_data_with_array_buffer_view(
        gl::ARRAY_BUFFER,
        &positions_array_buf_view,
        gl::STATIC_DRAW,
      );
    }

    Ok(buffer)
  }

  pub fn compile_shader(&self, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = self
      .context
      .create_shader(shader_type)
      .ok_or_else(|| String::from("Unable to create shader object"))?;
    self.context.shader_source(&shader, source);
    self.context.compile_shader(&shader);

    if self.context.get_shader_parameter(&shader, gl::COMPILE_STATUS).as_bool().unwrap_or(false) {
      Ok(shader)
    } else {
      Err(
        self
          .context
          .get_shader_info_log(&shader)
          .unwrap_or_else(|| String::from("Unknown error creating shader")),
      )
    }
  }

  pub fn link_program(
    &self,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
  ) -> Result<WebGlProgram, String> {
    let program = self
      .context
      .create_program()
      .ok_or_else(|| String::from("Unable to create shader object"))?;

    self.context.attach_shader(&program, vert_shader);
    self.context.attach_shader(&program, frag_shader);
    self.context.link_program(&program);

    if self.context.get_program_parameter(&program, gl::LINK_STATUS).as_bool().unwrap_or(false) {
      Ok(program)
    } else {
      Err(
        self
          .context
          .get_program_info_log(&program)
          .unwrap_or_else(|| String::from("Unknown error creating program object")),
      )
    }
  }

  pub fn setup_image(&self, width: i32, height: i32) {
    self.context.active_texture(gl::TEXTURE0);
    self.context.bind_texture(gl::TEXTURE_2D, Some(&self.texture));

    self.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    self.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    self.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    self.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    self.context.tex_storage_2d(gl::TEXTURE_2D, 1, gl::RGBA8, width, height);
  }
}

impl Image {
  pub fn load(&self, context: &Context, x: i32, y: i32) {
    context
      .context
      .tex_sub_image_2d_with_u32_and_u32_and_html_image_element(
        gl::TEXTURE_2D,
        0,
        x,
        y,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        &self.image.borrow(),
      )
      .unwrap();
  }

  pub fn width(&self) -> i32 { self.image.borrow().width() as i32 }
  pub fn height(&self) -> i32 { self.image.borrow().height() as i32 }
}
