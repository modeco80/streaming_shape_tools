
use serde::{Deserialize,Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ManifestFrame {
    pub path: String,
    pub frame_name: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ManifestRoot {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<ManifestFrame>
}