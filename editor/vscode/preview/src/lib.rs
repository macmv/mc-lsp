use wasm_bindgen::prelude::*;

mod render;

use render::Render;

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
  let render = Render::new()?;

  render.draw();

  Ok(())
}
