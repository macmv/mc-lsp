use crate::json;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  type VsMessage;

  #[wasm_bindgen(method, getter)]
  fn data(this: &VsMessage) -> JsValue;
}

#[derive(Deserialize, Debug)]
pub enum Message {
  RenderModel { model: json::Model },
}

pub fn listen(mut f: impl FnMut(Message) + 'static) {
  let closure = Closure::wrap(Box::new(move |event: VsMessage| {
    let message = event.data();

    match serde_wasm_bindgen::from_value::<Message>(message) {
      Ok(message) => f(message),
      Err(e) => error!("{:?}", e),
    }
  }) as Box<dyn FnMut(VsMessage)>);

  let window = web_sys::window().unwrap();

  window.add_event_listener_with_callback("message", closure.as_ref().unchecked_ref()).unwrap();

  closure.forget();
}
