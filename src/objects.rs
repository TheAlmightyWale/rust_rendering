use crate::properties::Color;
use crate::properties::Material;
use crate::serialization_defs::Vector3Def;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Sphere {
    #[serde(with = "Vector3Def")]
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn get_color(self) -> Color<u8> {
        match self.material {
            Material::Matte { color } => color,
            Material::Specular { color, specular: _ } => color,
        }
    }
}
