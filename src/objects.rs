use quaternion_core::{RotationSequence, RotationType};

#[derive(Debug, Default, Clone, Copy)]
pub struct Translation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl ToString for Translation {
    fn to_string(&self) -> String {
        format!("translation: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct Rotation {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}
impl ToString for Rotation {
    fn to_string(&self) -> String {
        let quaternion = quaternion_core::from_euler_angles(
            RotationType::Intrinsic,
            RotationSequence::YZX,
            [
                to_radians(self.pitch),
                to_radians(self.yaw),
                to_radians(self.roll),
            ],
        );
        format!(
            "left_rotation: [{}f,{}f,{}f,{}f]",
            quaternion.0, quaternion.1[0], quaternion.1[1], quaternion.1[2],
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl ToString for Scale {
    fn to_string(&self) -> String {
        format!("scale: [{}f,{}f,{}f]", self.x, self.y, self.z)
    }
}
#[derive(Debug, Default, Clone, Copy)]
pub struct Transformation {
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}
impl Transformation {
    pub fn with_translation(&self, translation: Translation) -> Self {
        Transformation {
            translation,
            ..*self
        }
    }
    pub fn with_rotation(&self, rotation: Rotation) -> Self {
        Transformation { rotation, ..*self }
    }
    pub fn with_scale(&self, scale: Scale) -> Self {
        Transformation { scale, ..*self }
    }
}

pub struct Entity {
    pub name: String,
    pub transformation: Transformation,
}
impl From<String> for Entity {
    fn from(value: String) -> Self {
        Self {
            name: value,
            transformation: Transformation::default(),
        }
    }
}

fn to_radians(degrees: f32) -> f32 {
    degrees * std::f32::consts::PI / 180.0
}
