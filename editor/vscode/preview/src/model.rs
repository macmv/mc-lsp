use std::collections::HashMap;

#[derive(Default)]
pub struct Buffers {
  pub pos:    Vec<[f32; 3]>,
  pub uv:     Vec<[f32; 2]>,
  pub normal: Vec<[f32; 3]>,
}

struct Builder<'a> {
  buffers: &'a mut Buffers,
  uv_map:  &'a HashMap<&'a str, (f64, f64, f64, f64)>,
}

pub fn render(model: &mc_message::Model, uv_map: &HashMap<&str, (f64, f64, f64, f64)>) -> Buffers {
  let mut buffers = Buffers::default();

  let mut builder = Builder { buffers: &mut buffers, uv_map };
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
  fn render_element(&mut self, element: &mc_message::Element) {
    for dir in Dir::all() {
      if let Some(face) = face_at_dir(&element.faces, dir) {
        self.render_face(element, &face, dir);
      }
    }
  }

  fn render_face(&mut self, element: &mc_message::Element, face: &mc_message::Face, dir: Dir) {
    // TODO: Replace with an error texture.
    let Some(ref tex) = face.texture else { return };
    let (u_min, v_min, u_width, v_height) = self.uv_map[tex.as_str()];

    let u0 = (face.uv[0] / 16.0) * u_width + u_min;
    let v0 = (face.uv[1] / 16.0) * v_height + v_min;
    let u1 = (face.uv[2] / 16.0) * u_width + u_min;
    let v1 = (face.uv[3] / 16.0) * v_height + v_min;

    let p0 = element.from;
    let p1 = element.to;
    match dir {
      Dir::Up => {
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
      }
      Dir::Down => {
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p0.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
      }
      Dir::South => {
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p1.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p1.z }, u0, v1);
      }
      Dir::North => {
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p0.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u1, v1);
      }
      Dir::East => {
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p0.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p0.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p1.y, z: p1.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p1.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p1.x, y: p0.y, z: p0.z }, u1, v1);
      }
      Dir::West => {
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p0.z }, u0, v0);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p0.z }, u0, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p0.y, z: p1.z }, u1, v1);
        self.vert(element, dir, mc_message::Pos { x: p0.x, y: p1.y, z: p1.z }, u1, v0);
      }
    }
  }

  fn vert(
    &mut self,
    element: &mc_message::Element,
    dir: Dir,
    mut p: mc_message::Pos,
    u: f64,
    v: f64,
  ) {
    let mut normal = dir.as_normal();

    if let Some(r) = &element.rotation {
      rotate(&r, &mut p);
      rotate(&r, &mut normal);
      if r.rescale {
        let v = (r.angle / 2.0 / 180.0 * std::f64::consts::PI).cos();
        p = match r.axis {
          mc_message::Axis::X => p / mc_message::Pos { x: 1.0, y: v, z: v },
          mc_message::Axis::Y => p / mc_message::Pos { x: v, y: 1.0, z: v },
          mc_message::Axis::Z => p / mc_message::Pos { x: v, y: v, z: 1.0 },
        };
        normal = match r.axis {
          mc_message::Axis::X => normal / mc_message::Pos { x: 1.0, y: v, z: v },
          mc_message::Axis::Y => normal / mc_message::Pos { x: v, y: 1.0, z: v },
          mc_message::Axis::Z => normal / mc_message::Pos { x: v, y: v, z: 1.0 },
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

    self.buffers.pos.push([p.x as f32, p.y as f32, p.z as f32]);
    self.buffers.uv.push([u as f32, v as f32]);
    self.buffers.normal.push([normal.x as f32, normal.y as f32, normal.z as f32]);
    /*
      tint:     [(tint_r * 255.0) as u8, (tint_g * 255.0) as u8, (tint_b * 255.0) as u8],
      light:    (block_light * 15.0) as u8 | (((sky_light * 15.0) as u8) << 4),
    */
  }
}

impl Dir {
  fn as_normal(&self) -> mc_message::Pos {
    match self {
      Dir::North => mc_message::Pos { x: 0.0, y: 0.0, z: -16.0 },
      Dir::South => mc_message::Pos { x: 0.0, y: 0.0, z: 16.0 },
      Dir::East => mc_message::Pos { x: 16.0, y: 0.0, z: 0.0 },
      Dir::West => mc_message::Pos { x: -16.0, y: 0.0, z: 0.0 },
      Dir::Up => mc_message::Pos { x: 0.0, y: 16.0, z: 0.0 },
      Dir::Down => mc_message::Pos { x: 0.0, y: -16.0, z: 0.0 },
    }
  }

  fn all() -> [Dir; 6] { [Dir::North, Dir::South, Dir::East, Dir::West, Dir::Up, Dir::Down] }
}

fn rotate(rotation: &mc_message::Rotation, p: &mut mc_message::Pos) {
  let c = (-rotation.angle / 180.0 * std::f64::consts::PI).cos();
  let s = (-rotation.angle / 180.0 * std::f64::consts::PI).sin();
  let diff = *p - rotation.origin;
  match rotation.axis {
    mc_message::Axis::X => {
      p.y = (diff.y * c - diff.z * s) + rotation.origin.y;
      p.z = (diff.z * c + diff.y * s) + rotation.origin.z;
    }
    mc_message::Axis::Y => {
      p.x = (diff.x * c - diff.z * s) + rotation.origin.x;
      p.z = (diff.z * c + diff.x * s) + rotation.origin.z;
    }
    mc_message::Axis::Z => {
      p.x = (diff.x * c - diff.y * s) + rotation.origin.x;
      p.y = (diff.y * c + diff.x * s) + rotation.origin.y;
    }
  }
}

fn face_at_dir(faces: &mc_message::Faces, dir: Dir) -> &Option<mc_message::Face> {
  match dir {
    Dir::North => &faces.north,
    Dir::South => &faces.south,
    Dir::East => &faces.east,
    Dir::West => &faces.west,
    Dir::Up => &faces.up,
    Dir::Down => &faces.down,
  }
}
