use crate::core::commands::generate::omni::{self, Modality, OmniRequest, OmniResult};
use crate::core::commands::generate::omni::dispatch::local_error;
use crate::core::commands::generate::fal_mesh::generate_mesh_via_fal;
use enums::common::generation_provider::GenerationProvider;
use tauri::AppHandle;

#[tauri::command]
pub async fn generate_mesh_command(request: OmniRequest, app: AppHandle) -> OmniResult {
  if request.provider == Some(GenerationProvider::Fal) {
    return generate_mesh_via_fal(request, &app).await;
  }
  if !request.uses_artcraft() {
    return Err(local_error("No direct provider adapter is configured for this modality."));
  }
  omni::generate(request, Modality::Mesh, &app).await
}
