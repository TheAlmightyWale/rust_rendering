//A scene is a description of objects that exist in the world, it handles loading of objects from a specified json file
// right now objects are limited to spheres and lights

use crate::lights::DirectionalLight;
use crate::objects::Sphere;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Scene {
    pub objects: Vec<Sphere>,
    pub lights: Vec<DirectionalLight>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
        }
    }

    pub fn load(scene_description_json: &str) -> Result<Scene> {
        let scene: Scene = serde_json::from_str(scene_description_json)?;
        Ok(scene)
    }

    //pub fn save(scene: &Scene) -> Result<&str> {}
}
