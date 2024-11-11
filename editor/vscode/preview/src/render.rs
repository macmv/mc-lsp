use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};

pub struct Render {
  context: WebGl2RenderingContext,
}

impl Render {
  pub fn new() -> Result<Self, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas.get_context("webgl2")?.unwrap().dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader =
      compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, include_str!("vert.glsl"))?;

    let frag_shader =
      compile_shader(&context, WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("frag.glsl"))?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    // A 1x1x1 cube.
    let vertices = [
      [0.0, 0.0, 0.0],
      [1.0, 0.0, 0.0],
      [1.0, 1.0, 0.0],
      [0.0, 1.0, 0.0],
      [0.0, 0.0, 1.0],
      [1.0, 0.0, 1.0],
      [1.0, 1.0, 1.0],
      [0.0, 1.0, 1.0],
    ];

    let indices: [u16; 6 * 6] = [
      0, 1, 3, 3, 1, 2, // face 1
      1, 5, 2, 2, 5, 6, // face 2
      5, 4, 6, 6, 4, 7, // face 3
      4, 0, 7, 7, 0, 3, // face 4
      3, 2, 7, 7, 2, 6, // face 5
      4, 5, 0, 0, 5, 1, // face 6
    ];

    let vao = context.create_vertex_array().ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao));

    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
    unsafe {
      let positions_array_buf_view = js_sys::Float32Array::view(bytemuck::cast_slice(&vertices));

      context.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ARRAY_BUFFER,
        &positions_array_buf_view,
        WebGl2RenderingContext::STATIC_DRAW,
      );
    }

    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
    unsafe {
      let indices_buf_view = js_sys::Uint16Array::view(bytemuck::cast_slice(&indices));

      context.buffer_data_with_array_buffer_view(
        WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
        &indices_buf_view,
        WebGl2RenderingContext::STATIC_DRAW,
      );
    }

    let position_attribute_location = context.get_attrib_location(&program, "pos");
    context.vertex_attrib_pointer_with_i32(
      position_attribute_location as u32,
      3,
      WebGl2RenderingContext::FLOAT,
      false,
      0,
      0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);

    // context.bind_vertex_array(Some(&vao));

    Ok(Render { context })
  }

  pub fn draw(&self) {
    self.context.clear_color(0.0, 0.0, 0.0, 1.0);
    self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    self.context.draw_elements_with_i32(
      WebGl2RenderingContext::TRIANGLES,
      6 * 2,
      WebGl2RenderingContext::UNSIGNED_SHORT,
      0,
    );
  }
}

pub fn compile_shader(
  context: &WebGl2RenderingContext,
  shader_type: u32,
  source: &str,
) -> Result<WebGlShader, String> {
  let shader = context
    .create_shader(shader_type)
    .ok_or_else(|| String::from("Unable to create shader object"))?;
  context.shader_source(&shader, source);
  context.compile_shader(&shader);

  if context
    .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(
      context
        .get_shader_info_log(&shader)
        .unwrap_or_else(|| String::from("Unknown error creating shader")),
    )
  }
}

pub fn link_program(
  context: &WebGl2RenderingContext,
  vert_shader: &WebGlShader,
  frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
  let program =
    context.create_program().ok_or_else(|| String::from("Unable to create shader object"))?;

  context.attach_shader(&program, vert_shader);
  context.attach_shader(&program, frag_shader);
  context.link_program(&program);

  if context
    .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
    .as_bool()
    .unwrap_or(false)
  {
    Ok(program)
  } else {
    Err(
      context
        .get_program_info_log(&program)
        .unwrap_or_else(|| String::from("Unknown error creating program object")),
    )
  }
}
