//Defines the different forms of lights that a scene can possess
use crate::serialization_defs::Vector3Def;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DirectionalLight {
    #[serde(with = "Vector3Def")]
    pub direction: cgmath::Vector3<f32>,
}
