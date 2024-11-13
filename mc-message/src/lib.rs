use std::fmt;

use serde::{
  de::{self, Visitor},
  ser::SerializeSeq,
  Deserialize, Serialize,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Model {
  pub elements: Vec<Element>,
}

impl Model {
  pub fn textures(&self) -> impl Iterator<Item = &str> {
    self.elements.iter().flat_map(|e| e.faces.textures())
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Element {
  pub from:     Pos,
  pub to:       Pos,
  pub faces:    Faces,
  pub rotation: Option<Rotation>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Faces {
  pub north: Option<Face>,
  pub south: Option<Face>,
  pub east:  Option<Face>,
  pub west:  Option<Face>,
  pub up:    Option<Face>,
  pub down:  Option<Face>,
}

impl Faces {
  pub fn textures(&self) -> impl Iterator<Item = &str> {
    self
      .north
      .iter()
      .chain(self.south.iter())
      .chain(self.east.iter())
      .chain(self.west.iter())
      .chain(self.up.iter())
      .chain(self.down.iter())
      .map(|f| f.texture.as_str())
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Face {
  pub uv:      [f64; 4],
  pub texture: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Rotation {
  pub origin:  Pos,
  pub axis:    Axis,
  pub angle:   f64,
  #[serde(default = "make_false")]
  pub rescale: bool,
}

fn make_false() -> bool { false }

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Axis {
  #[serde(rename = "x")]
  X,
  #[serde(rename = "y")]
  Y,
  #[serde(rename = "z")]
  Z,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Serialize for Pos {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: serde::Serializer,
  {
    let mut seq = serializer.serialize_seq(Some(3))?;
    seq.serialize_element(&self.x)?;
    seq.serialize_element(&self.y)?;
    seq.serialize_element(&self.z)?;
    seq.end()
  }
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
        let x = seq.next_element::<f64>()?.ok_or(de::Error::missing_field("x"))?;
        let y = seq.next_element::<f64>()?.ok_or(de::Error::missing_field("y"))?;
        let z = seq.next_element::<f64>()?.ok_or(de::Error::missing_field("z"))?;

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
