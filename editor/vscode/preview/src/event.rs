use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  type VsMessage;

  #[wasm_bindgen(method, getter)]
  fn data(this: &VsMessage) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
  type MouseMove;

  #[wasm_bindgen(method, getter)]
  fn movementX(this: &MouseMove) -> f32;
  #[wasm_bindgen(method, getter)]
  fn movementY(this: &MouseMove) -> f32;
}

#[wasm_bindgen]
extern "C" {
  type MouseDown;
}

#[wasm_bindgen]
extern "C" {
  type MouseUp;
}

#[derive(Deserialize, Debug)]
pub enum Message {
  RenderModel { model: mc_message::Model },
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

  window.set_onmessage(Some(closure.as_ref().unchecked_ref()));

  closure.forget();
}

pub fn on_mouse_move(mut f: impl FnMut(f32, f32) + 'static) {
  let closure = Closure::wrap(Box::new(move |event: MouseMove| {
    f(event.movementX(), event.movementY());
  }) as Box<dyn FnMut(MouseMove)>);

  let window = web_sys::window().unwrap();

  window.set_onmousemove(Some(closure.as_ref().unchecked_ref()));

  closure.forget();
}

pub fn on_mouse_down(mut f: impl FnMut() + 'static) {
  let closure = Closure::wrap(Box::new(move |_: MouseDown| {
    f();
  }) as Box<dyn FnMut(MouseDown)>);

  let window = web_sys::window().unwrap();

  window.set_onmousedown(Some(closure.as_ref().unchecked_ref()));

  closure.forget();
}

pub fn on_mouse_up(mut f: impl FnMut() + 'static) {
  let closure = Closure::wrap(Box::new(move |_: MouseUp| {
    f();
  }) as Box<dyn FnMut(MouseUp)>);

  let window = web_sys::window().unwrap();

  window.set_onmouseup(Some(closure.as_ref().unchecked_ref()));

  closure.forget();
}
