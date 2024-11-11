use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  rc::Rc,
};

use wasm_bindgen::prelude::*;
use web_sys::{
  HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlTexture,
  WebGlUniformLocation,
};

#[derive(Clone)]
pub struct Render {
  pub context: Context,

  proj_uniform_location:  Option<WebGlUniformLocation>,
  view_uniform_location:  Option<WebGlUniformLocation>,
  model_uniform_location: Option<WebGlUniformLocation>,

  tex_uniform_location: Option<WebGlUniformLocation>,
}

#[derive(Clone)]
pub struct Context {
  context: WebGl2RenderingContext,
}

pub struct Image {
  image:   Rc<RefCell<HtmlImageElement>>,
  texture: WebGlTexture,
}

impl Render {
  pub fn new() -> Result<Self, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

    let context = Context { context };

    let vert_shader =
      context.compile_shader(WebGl2RenderingContext::VERTEX_SHADER, include_str!("vert.glsl"))?;

    let frag_shader =
      context.compile_shader(WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("frag.glsl"))?;
    let program = context.link_program(&vert_shader, &frag_shader)?;
    context.context.use_program(Some(&program));

    // A 1x1x1 cube.
    let vertices: [[f32; 3]; 8] = [
      [0.0, 0.0, 0.0],
      [1.0, 0.0, 0.0],
      [1.0, 1.0, 0.0],
      [0.0, 1.0, 0.0],
      [0.0, 0.0, 1.0],
      [1.0, 0.0, 1.0],
      [1.0, 1.0, 1.0],
      [0.0, 1.0, 1.0],
    ];

    let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let normals: [[f32; 3]; 6] = [
      [0.0, 0.0, 1.0],
      [1.0, 0.0, 0.0],
      [0.0, 0.0, -1.0],
      [-1.0, 0.0, 0.0],
      [0.0, 1.0, 0.0],
      [0.0, -1.0, 0.0],
    ];

    let indices: [u16; 6 * 6] = [
      0, 1, 3, 3, 1, 2, // face 1
      1, 5, 2, 2, 5, 6, // face 2
      5, 4, 6, 6, 4, 7, // face 3
      4, 0, 7, 7, 0, 3, // face 4
      3, 2, 7, 7, 2, 6, // face 5
      4, 5, 0, 0, 5, 1, // face 6
    ];

    let uv_indices = [3, 2, 0, 0, 2, 1];

    let mut vert = [[0.0, 0.0, 0.0]; 6 * 6];
    let mut uv = [[0.0, 0.0]; 6 * 6];
    let mut normal = [[0.0, 0.0, 0.0]; 6 * 6];

    for i in 0..36 {
      vert[i] = vertices[indices[i] as usize];
      uv[i] = uvs[uv_indices[i % 6]];
      normal[i] = normals[indices[i / 6] as usize];
    }

    let vao =
      context.context.create_vertex_array().ok_or("Could not create vertex array object")?;
    context.context.bind_vertex_array(Some(&vao));

    context.create_f32_buffer(bytemuck::cast_slice(&vert))?;
    let pos_attribute_location = context.context.get_attrib_location(&program, "pos");
    context.context.vertex_attrib_pointer_with_i32(
      pos_attribute_location as u32,
      3,
      WebGl2RenderingContext::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(pos_attribute_location as u32);

    context.create_f32_buffer(bytemuck::cast_slice(&uv))?;
    let uv_attribute_location = context.context.get_attrib_location(&program, "uv");
    context.context.vertex_attrib_pointer_with_i32(
      uv_attribute_location as u32,
      2,
      WebGl2RenderingContext::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(uv_attribute_location as u32);

    context.create_f32_buffer(bytemuck::cast_slice(&normal))?;
    let normal_attribute_location = context.context.get_attrib_location(&program, "normal");
    context.context.vertex_attrib_pointer_with_i32(
      normal_attribute_location as u32,
      3,
      WebGl2RenderingContext::FLOAT,
      false,
      0,
      0,
    );
    context.context.enable_vertex_attrib_array(normal_attribute_location as u32);

    context.context.enable(WebGl2RenderingContext::DEPTH_TEST);

    Ok(Render {
      proj_uniform_location: context.context.get_uniform_location(&program, "proj"),
      view_uniform_location: context.context.get_uniform_location(&program, "view"),
      model_uniform_location: context.context.get_uniform_location(&program, "model"),
      tex_uniform_location: context.context.get_uniform_location(&program, "tex"),
      context,
    })
  }

  pub fn load_images(
    &self,
    paths: &HashSet<String>,
    on_load: impl FnOnce(&HashMap<String, Image>) + 'static,
  ) {
    let mut images = HashMap::new();
    let done = Rc::new(RefCell::new(HashMap::new()));
    let total = paths.len();
    let on_load = Rc::new(RefCell::new(Some(on_load)));
    for path in paths {
      let image = HtmlImageElement::new().unwrap();
      image.set_src(&path);

      let rc = Rc::new(RefCell::new(image));

      let image =
        Image { image: rc.clone(), texture: self.context.context.create_texture().unwrap() };
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

  pub fn draw(&self, proj: &[f32], view: &[f32], model: &[f32]) {
    self.context.context.clear_color(0.0, 0.0, 0.0, 1.0);
    self.context.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

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

    self.context.context.uniform1i(self.tex_uniform_location.as_ref(), 0);

    self.context.context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6 * 6);
  }

  pub fn setup_loop(self, mut f: impl FnMut(&Render) + 'static) {
    // Don't look too close, you're eyes might fall out.
    let render_func: Rc<RefCell<Option<Closure<_>>>> = Rc::new(RefCell::new(None));
    let render_func_2 = render_func.clone();

    *render_func.borrow_mut() = Some(Closure::new(move || {
      f(&self);

      let window = web_sys::window().unwrap();
      window
        .request_animation_frame(render_func_2.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
    }));

    let window = web_sys::window().unwrap();
    window
      .request_animation_frame(render_func.borrow().as_ref().unwrap().as_ref().unchecked_ref())
      .unwrap();
  }
}

impl Context {
  pub fn create_f32_buffer(&self, data: &[f32]) -> Result<WebGlBuffer, String> {
    let buffer = self.context.create_buffer().ok_or("Failed to create buffer")?;
    self.context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    unsafe {
      let positions_array_buf_view = js_sys::Float32Array::view(data);

      self.context.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &positions_array_buf_view,
        WebGl2RenderingContext::STATIC_DRAW,
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

    if self
      .context
      .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
      .as_bool()
      .unwrap_or(false)
    {
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

    if self
      .context
      .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
      .as_bool()
      .unwrap_or(false)
    {
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
}

impl Image {
  pub fn load(&self, context: &Context) {
    use WebGl2RenderingContext as gl;

    context.context.active_texture(gl::TEXTURE0);
    context.context.bind_texture(gl::TEXTURE_2D, Some(&self.texture));

    context.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    context.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    context.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
    context.context.tex_parameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

    context
      .context
      .tex_image_2d_with_u32_and_u32_and_html_image_element(
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        &self.image.borrow(),
      )
      .unwrap();
  }

  pub fn bind(&self, context: &Context) {
    use WebGl2RenderingContext as gl;

    context.context.active_texture(gl::TEXTURE0);
    context.context.bind_texture(gl::TEXTURE_2D, Some(&self.texture));
  }
}
