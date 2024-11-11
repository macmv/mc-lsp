use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  type VsMessage;

  #[wasm_bindgen(method, getter)]
  fn data(this: &VsMessage) -> JsValue;
}

#[derive(Deserialize)]
enum Message {
  RenderModel(Model),
}

#[derive(Deserialize)]
struct Model {
  foo: i32,
}

pub fn listen() -> Result<(), JsValue> {
  let window = web_sys::window().unwrap();
  window.add_event_listener_with_callback(
    "message",
    Closure::wrap(Box::new(move |event: VsMessage| {
      // This is quite dumb, but its better than needing to re-encode JSON to
      // decode it with serde.
      let message = event.data();

      let _message = serde_wasm_bindgen::from_value::<Message>(message);
    }) as Box<dyn FnMut(VsMessage)>)
    .as_ref()
    .unchecked_ref(),
  )?;

  Ok(())
}
