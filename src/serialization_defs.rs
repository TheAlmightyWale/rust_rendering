use cgmath::Vector3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vector3")]
pub struct Vector3Def<S> {
    x: S,
    y: S,
    z: S,
}
