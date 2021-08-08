use crate::serialization_defs::Vector3Def;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Sphere {
    #[serde(with = "Vector3Def")]
    pub center: cgmath::Vector3<f32>,
    pub radius: f32,
    pub color: [u8; 4],
}
