use carla::sensor::data::{Color, Image as ImageEvent};
use ndarray::{Array2, ArrayView2};
use serde::{Deserialize, Serialize};

/// Remote schema for the foreign element type
#[derive(Serialize, Deserialize)]
#[serde(remote = "carla::sensor::data::Color")]
struct ColorRemote {
    b: u8,
    g: u8,
    r: u8,
    a: u8,
}

// ---------------------------------------------------------------------
// Borrowed, serialize-only for ArrayView2<Color>
// ---------------------------------------------------------------------
mod arrayview2_color_remote {
    use super::*;
    use serde::Serialize;
    use serde::ser::{SerializeSeq, Serializer};

    // Serialize &Color via the remote impl
    struct ColorAsRemote<'a>(&'a Color);
    impl<'a> Serialize for ColorAsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::ColorRemote::serialize(self.0, s)
        }
    }

    // Helper to serialize one row without allocating
    struct Row<'a>(ndarray::ArrayView1<'a, Color>);
    impl<'a> Serialize for Row<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut inner = s.serialize_seq(Some(self.0.len()))?;
            for c in self.0.iter() {
                inner.serialize_element(&ColorAsRemote(c))?;
            }
            inner.end()
        }
    }

    // Serialize ArrayView2<Color> as a seq of rows (serialize-only)
    pub fn serialize<S: Serializer>(arr: &ArrayView2<Color>, s: S) -> Result<S::Ok, S::Error> {
        let (h, _) = arr.dim();
        let mut outer = s.serialize_seq(Some(h))?;
        for row in arr.rows() {
            outer.serialize_element(&Row(row))?;
        }
        outer.end()
    }
}

/// Borrowed, zero-copy serializer for Image
#[derive(Serialize)]
pub struct ImageEventSerBorrowed<'a> {
    pub height: usize,
    pub width: usize,
    pub len: usize,
    pub is_empty: bool,
    pub fov_angle: f32,
    #[serde(with = "self::arrayview2_color_remote")]
    pub array: ArrayView2<'a, Color>,
}

impl<'a> From<&'a ImageEvent> for ImageEventSerBorrowed<'a> {
    fn from(value: &'a ImageEvent) -> Self {
        Self {
            height: value.height(),
            width: value.width(),
            len: value.len(),
            is_empty: value.is_empty(),
            fov_angle: value.fov_angle(),
            array: value.as_array(), // borrow, zero-copy
        }
    }
}

// ---------------------------------------------------------------------
// Owned, round-trippable for Array2<Color>
// ---------------------------------------------------------------------
mod array2_color_remote {
    use super::*;
    use serde::de::{self, SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;

    // Serialize &Color via the remote impl
    struct ColorAsRemote<'a>(&'a Color);
    impl<'a> Serialize for ColorAsRemote<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            super::ColorRemote::serialize(self.0, s)
        }
    }

    // Deserialize Color via the remote impl
    struct ColorFromRemote(Color);
    impl<'de> Deserialize<'de> for ColorFromRemote {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            super::ColorRemote::deserialize(d).map(ColorFromRemote)
        }
    }

    // Helper to serialize one row without allocating
    struct Row<'a>(ndarray::ArrayView1<'a, Color>);
    impl<'a> Serialize for Row<'a> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut inner = s.serialize_seq(Some(self.0.len()))?;
            for c in self.0.iter() {
                inner.serialize_element(&ColorAsRemote(c))?;
            }
            inner.end()
        }
    }

    // Serialize Array2<Color> as a seq of rows
    pub fn serialize<S: Serializer>(arr: &Array2<Color>, s: S) -> Result<S::Ok, S::Error> {
        let (h, _) = arr.dim();
        let mut outer = s.serialize_seq(Some(h))?;
        for row in arr.rows() {
            outer.serialize_element(&Row(row))?;
        }
        outer.end()
    }

    // Deserialize Vec<Vec<ColorRemote>> back into Array2<Color>
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Array2<Color>, D::Error> {
        struct Outer;
        impl<'de> Visitor<'de> for Outer {
            type Value = Array2<Color>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<Vec<Color>> with equal-length rows")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut outer: A) -> Result<Self::Value, A::Error> {
                let mut rows: Vec<Vec<Color>> = Vec::new();
                while let Some(inner) = outer.next_element::<Vec<ColorFromRemote>>()? {
                    rows.push(inner.into_iter().map(|x| x.0).collect());
                }
                let h = rows.len();
                let w = rows.get(0).map_or(0, |r| r.len());
                if w == 0 && h == 0 {
                    return Ok(Array2::from_shape_vec((0, 0), vec![]).unwrap());
                }
                for r in &rows {
                    if r.len() != w {
                        return Err(de::Error::custom("ragged 2D array"));
                    }
                }
                let flat: Vec<Color> = rows.into_iter().flatten().collect();
                ndarray::Array2::from_shape_vec((h, w), flat).map_err(de::Error::custom)
            }
        }
        d.deserialize_seq(Outer)
    }
}

/// Owned, round-trip serializer for Image
#[derive(Serialize, Deserialize)]
pub struct ImageEventSerDe {
    pub height: usize,
    pub width: usize,
    pub len: usize,
    pub is_empty: bool,
    pub fov_angle: f32,
    #[serde(with = "self::array2_color_remote")]
    pub array: Array2<Color>,
}

impl From<ImageEvent> for ImageEventSerDe {
    fn from(value: ImageEvent) -> Self {
        // Build an owned Array2<Color> without requiring Clone by copying fields (u8s)
        let view = value.as_array();
        let array: Array2<Color> = view.map(|c| Color {
            b: c.b,
            g: c.g,
            r: c.r,
            a: c.a,
        });

        Self {
            height: value.height(),
            width: value.width(),
            len: value.len(),
            is_empty: value.is_empty(),
            fov_angle: value.fov_angle(),
            array,
        }
    }
}
