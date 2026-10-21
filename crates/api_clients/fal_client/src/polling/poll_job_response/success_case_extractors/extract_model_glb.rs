use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct PollResponseModelGlbData {
  pub url: Option<String>,
  pub content_type: Option<String>,
  pub file_name: Option<String>,
  pub file_size: Option<u64>,
}

pub fn extract_model_glb(value: &Value) -> Option<PollResponseModelGlbData> {
  // Most fal 3D models return `model_glb`; Hunyuan3D 2.x returns `model_mesh`.
  let model_glb = value.get("model_glb").or_else(|| value.get("model_mesh"))?;
  serde_json::from_value(model_glb.clone()).ok()
}

#[cfg(test)]
mod model_mesh_tests {
  use super::extract_model_glb;

  #[test]
  fn hunyuan_model_mesh_is_extracted() {
    let value: serde_json::Value = serde_json::from_str(r#"{"model_mesh":{"url":"https://v3b.fal.media/files/b/mesh.glb","content_type":"model/gltf-binary","file_name":"mesh.glb","file_size":3253532},"seed":367190}"#).unwrap();
    let glb = extract_model_glb(&value).expect("should extract model_mesh");
    assert_eq!(glb.url.as_deref(), Some("https://v3b.fal.media/files/b/mesh.glb"));
  }
}
