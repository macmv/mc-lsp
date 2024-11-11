use std::{collections::HashMap, ops::Index};

use crate::json;

#[derive(Default)]
pub struct Buffers {
  pub pos:    Vec<[f32; 3]>,
  pub uv:     Vec<[f32; 2]>,
  pub normal: Vec<[f32; 3]>,
}

struct Builder<'a> {
  buffers: &'a mut Buffers,
  model:   &'a json::Model,
  uv_map:  &'a HashMap<&'a str, (f32, f32, f32, f32)>,
}

pub fn render(model: &json::Model, uv_map: &HashMap<&str, (f32, f32, f32, f32)>) -> Buffers {
  let mut buffers = Buffers::default();

  let mut builder = Builder { buffers: &mut buffers, model, uv_map };
  for element in model.elements.iter() {
    builder.render_element(element);
  }

  buffers
}

#[derive(Debug, Clone, Copy)]
enum Dir {
  North,
  South,
  East,
  West,
  Up,
  Down,
}

impl Builder<'_> {
  fn render_element(&mut self, element: &json::Element) {
    for dir in Dir::all() {
      if let Some(face) = &element.faces[dir] {
        self.render_face(element, face, dir);
      }
    }
  }

  fn render_face(&mut self, element: &json::Element, face: &json::Face, dir: Dir) {
    let Some(key) = face.texture.strip_prefix("#") else { return };
    // TODO: Pack in all the textures to an atlas.
    let Some(texture) = self.model.textures.get(key) else { return };

    let (u_min, v_min, u_width, v_height) = self.uv_map[texture.as_str()];

    let u0 = (face.uv[0] / 16.0) * u_width + u_min;
    let v0 = (face.uv[1] / 16.0) * v_height + v_min;
    let u1 = (face.uv[2] / 16.0) * u_width + u_min;
    let v1 = (face.uv[3] / 16.0) * v_height + v_min;

    let p0 = element.from;
    let p1 = element.to;
    match dir {
      Dir::Up => {
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
      }
      Dir::Down => {
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p0.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
      }
      Dir::South => {
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p1.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p1.z }, u0, v1);
      }
      Dir::North => {
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u1, v1);
      }
      Dir::East => {
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p0.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p0.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p1.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p1.x, y: p0.y, z: p0.z }, u1, v1);
      }
      Dir::West => {
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, json::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
      }
    }
  }

  fn vert(&mut self, element: &json::Element, dir: Dir, mut p: json::Pos, u: f32, v: f32) {
    let mut normal = dir.as_normal();

    if let Some(r) = &element.rotation {
      r.rotate(&mut p);
      r.rotate(&mut normal);
      if r.rescale {
        let v = (r.angle / 2.0 / 180.0 * std::f32::consts::PI).cos();
        p = match r.axis {
          json::Axis::X => p / json::Pos { x: 1.0, y: v, z: v },
          json::Axis::Y => p / json::Pos { x: v, y: 1.0, z: v },
          json::Axis::Z => p / json::Pos { x: v, y: v, z: 1.0 },
        };
        normal = match r.axis {
          json::Axis::X => normal / json::Pos { x: 1.0, y: v, z: v },
          json::Axis::Y => normal / json::Pos { x: v, y: 1.0, z: v },
          json::Axis::Z => normal / json::Pos { x: v, y: v, z: 1.0 },
        };
      }
    }

    p.x /= 16.0;
    p.y /= 16.0;
    p.z /= 16.0;
    normal.x /= 16.0;
    normal.y /= 16.0;
    normal.z /= 16.0;

    /*
    let mut tint_r = 1.0;
    let mut tint_g = 1.0;
    let mut tint_b = 1.0;

    if face.tintindex == Some(0) {
      (tint_r, tint_g, tint_b) = self.tint();
    }
    */

    self.buffers.pos.push([p.x, p.y, p.z]);
    self.buffers.uv.push([u, v]);
    self.buffers.normal.push([normal.x, normal.y, normal.z]);
    /*
      tint:     [(tint_r * 255.0) as u8, (tint_g * 255.0) as u8, (tint_b * 255.0) as u8],
      light:    (block_light * 15.0) as u8 | (((sky_light * 15.0) as u8) << 4),
    */
  }
}

impl Dir {
  fn as_normal(&self) -> json::Pos {
    match self {
      Dir::North => json::Pos { x: 0.0, y: 0.0, z: -16.0 },
      Dir::South => json::Pos { x: 0.0, y: 0.0, z: 16.0 },
      Dir::East => json::Pos { x: 16.0, y: 0.0, z: 0.0 },
      Dir::West => json::Pos { x: -16.0, y: 0.0, z: 0.0 },
      Dir::Up => json::Pos { x: 0.0, y: 16.0, z: 0.0 },
      Dir::Down => json::Pos { x: 0.0, y: -16.0, z: 0.0 },
    }
  }

  fn all() -> [Dir; 6] { [Dir::North, Dir::South, Dir::East, Dir::West, Dir::Up, Dir::Down] }
}

impl json::Rotation {
  fn rotate(&self, p: &mut json::Pos) {
    let c = (-self.angle / 180.0 * std::f32::consts::PI).cos();
    let s = (-self.angle / 180.0 * std::f32::consts::PI).sin();
    let diff = *p - self.origin;
    match self.axis {
      json::Axis::X => {
        p.y = (diff.y * c - diff.z * s) + self.origin.y;
        p.z = (diff.z * c + diff.y * s) + self.origin.z;
      }
      json::Axis::Y => {
        p.x = (diff.x * c - diff.z * s) + self.origin.x;
        p.z = (diff.z * c + diff.x * s) + self.origin.z;
      }
      json::Axis::Z => {
        p.x = (diff.x * c - diff.y * s) + self.origin.x;
        p.y = (diff.y * c + diff.x * s) + self.origin.y;
      }
    }
  }
}

impl Index<Dir> for json::Faces {
  type Output = Option<json::Face>;

  fn index(&self, dir: Dir) -> &Self::Output {
    match dir {
      Dir::North => &self.north,
      Dir::South => &self.south,
      Dir::East => &self.east,
      Dir::West => &self.west,
      Dir::Up => &self.up,
      Dir::Down => &self.down,
    }
  }
}
