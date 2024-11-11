use serde::{
  de::{self, Visitor},
  Deserialize,
};
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Debug)]
pub struct Model {
  pub textures: HashMap<String, String>,
  pub elements: Vec<Element>,
}

#[derive(Deserialize, Debug)]
pub struct Element {
  pub from:     Pos,
  pub to:       Pos,
  pub faces:    Faces,
  pub rotation: Option<Rotation>,
}

#[derive(Deserialize, Debug)]
pub struct Faces {
  pub north: Option<Face>,
  pub south: Option<Face>,
  pub east:  Option<Face>,
  pub west:  Option<Face>,
  pub up:    Option<Face>,
  pub down:  Option<Face>,
}

#[derive(Deserialize, Debug)]
pub struct Face {
  pub uv:      [f32; 4],
  pub texture: String,
}

#[derive(Deserialize, Debug)]
pub struct Rotation {
  pub origin:  Pos,
  pub axis:    Axis,
  pub angle:   f32,
  #[serde(default = "make_false")]
  pub rescale: bool,
}

fn make_false() -> bool { false }

#[derive(Deserialize, Debug)]
pub enum Axis {
  #[serde(rename = "x")]
  X,
  #[serde(rename = "y")]
  Y,
  #[serde(rename = "z")]
  Z,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pos {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

impl<'de> Deserialize<'de> for Pos {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: serde::Deserializer<'de>,
  {
    struct PosVisitor;

    impl<'de> Visitor<'de> for PosVisitor {
      type Value = Pos;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a position")
      }

      fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
      where
        A: de::SeqAccess<'de>,
      {
        let x = seq.next_element::<f32>()?.ok_or(de::Error::missing_field("x"))?;
        let y = seq.next_element::<f32>()?.ok_or(de::Error::missing_field("y"))?;
        let z = seq.next_element::<f32>()?.ok_or(de::Error::missing_field("z"))?;

        Ok(Pos { x, y, z })
      }
    }

    deserializer.deserialize_seq(PosVisitor)
  }
}

impl std::ops::Add for Pos {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    Pos { x: self.x + rhs.x, y: self.y + rhs.y, z: self.z + rhs.z }
  }
}
impl std::ops::Sub for Pos {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    Pos { x: self.x - rhs.x, y: self.y - rhs.y, z: self.z - rhs.z }
  }
}
impl std::ops::Div for Pos {
  type Output = Self;

  fn div(self, rhs: Pos) -> Self::Output {
    Pos { x: self.x / rhs.x, y: self.y / rhs.y, z: self.z / rhs.z }
  }
}
