//Defines the different forms of lights that a scene can possess
use crate::properties::Color;
use crate::serialization_defs::Vector3Def;
use serde::{Deserialize, Serialize};

//Intensities should probably be only 3 channel, as how can you somehow have more / less transparent light??
#[derive(Serialize, Deserialize, Debug)]
pub enum Light {
    Directional {
        #[serde(with = "Vector3Def")]
        direction: cgmath::Vector3<f32>,
        intensity: Color<f32>,
    },
    Ambient {
        intensity: Color<f32>,
    },
    Point {
        #[serde(with = "Vector3Def")]
        position: cgmath::Vector3<f32>,
        intensity: Color<f32>,
    },
}
