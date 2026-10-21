use artcraft_router::api::image_list_ref::ImageListRef;
use artcraft_router::api::router_mesh_model::RouterMeshModel;
use artcraft_router::api::router_provider::RouterProvider;
use artcraft_router::client::router_client::RouterClient;
use artcraft_router::client::router_fal_client::RouterFalClient;
use artcraft_router::generate::generate_mesh::generate_mesh_request_builder::GenerateMeshRequestBuilder;
use artcraft_router::generate::generate_mesh::mesh_generation_draft_or_request::MeshGenerationDraftOrRequest;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{error, info, warn};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};
use tokens::tokens::media_files::MediaFileToken;

use crate::core::commands::enqueue::generate_error::GenerateError;
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::map_media_files_to_urls::map_media_file_tokens_to_cdn_urls;
use crate::core::commands::generate::generate_video::fal::handle_fal_video::load_fal_api_key;
use crate::core::commands::generate::omni::dispatch::{local_error, OmniResponse, OmniResult};
use crate::core::commands::generate::omni::OmniRequest;
use crate::core::events::generation_events::common::GenerationModel;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;
use crate::core::state::task_database::TaskDatabase;

/// Image-to-3D straight to fal with the user's own API key.
/// The finished GLB is picked up by the fal task polling thread.
pub async fn generate_mesh_via_fal(request: OmniRequest, app: &AppHandle) -> OmniResult {
  let success = enqueue(&request, app).await.map_err(|err| {
    error!("fal image-to-3D failed: {:?}", err);
    local_error(format!("{:?}", err))
  })?;

  let task_database = app.state::<TaskDatabase>();
  if let Err(err) = success
    .insert_into_task_database_with_frontend_payload(
      &task_database,
      request.frontend_caller,
      request.frontend_subscriber_id.as_deref(),
      request.frontend_subscriber_payload.as_deref(),
    )
    .await
  {
    error!("Failed to create task in database: {:?}", err);
  }

  Ok(OmniResponse(json!({})).into())
}

async fn enqueue(request: &OmniRequest, app: &AppHandle) -> Result<TaskEnqueueSuccess, GenerateError> {
  let (router_model, generation_model) = map_model(request.model())?;
  let api_key = load_fal_api_key(&app.state::<ProviderCredentialLoadingCache>())?;
  let api_host = app.state::<AppEnvConfigs>().storyteller_host.clone();

  // The 3D page sends `reference_image_media_tokens`; older callers send `image_media_token`.
  let image_token = request.fields.get("reference_image_media_tokens")
    .and_then(Value::as_array)
    .and_then(|tokens| tokens.first())
    .and_then(Value::as_str)
    .or_else(|| request.fields.get("image_media_token").and_then(Value::as_str));
  let prompt = request.fields.get("prompt").and_then(Value::as_str).map(str::to_owned);

  let reference_images = match image_token {
    Some(token) => {
      let urls = map_media_file_tokens_to_cdn_urls(&[MediaFileToken::new_from_str(token)], &api_host).await?;
      Some(ImageListRef::Urls(urls))
    }
    None => None,
  };

  if reference_images.is_none() && prompt.is_none() {
    return Err(GenerateError::NotYetImplemented("3D generation needs an image or a prompt".to_string()));
  }

  let builder = GenerateMeshRequestBuilder {
    model: router_model,
    provider: RouterProvider::Fal,
    prompt,
    reference_images,
    ..Default::default()
  };

  let request_to_send = match builder.build2()? {
    MeshGenerationDraftOrRequest::Request(r) => r,
    _ => return Err(GenerateError::NotYetImplemented("fal 3D should not produce a draft request".to_string())),
  };

  let client = RouterClient::Fal(RouterFalClient::new_polling_only_from_raw_key(&api_key));
  info!("Executing fal image-to-3D: model={:?}", router_model);

  let response = request_to_send.send_request(&client).await.map_err(|err| {
    warn!("fal image-to-3D request failed: {:?}", err);
    GenerateError::from(err)
  })?;

  let fal_payload = response.get_fal_payload().ok_or(GenerateError::ResponseHadNoJobTokens)?;
  let provider_job_id = fal_payload.request_id
    .or(fal_payload.gateway_request_id)
    .ok_or(GenerateError::ResponseHadNoJobTokens)?;

  Ok(TaskEnqueueSuccess {
    task_type: TaskType::ObjectGeneration,
    model: Some(generation_model),
    provider: GenerationProvider::Fal,
    provider_job_id: Some(provider_job_id),
    maybe_queue_status_url: fal_payload.maybe_status_url,
    maybe_queue_response_url: fal_payload.maybe_response_url,
    maybe_prompt_token: None,
  })
}

fn map_model(model: Option<&str>) -> Result<(RouterMeshModel, GenerationModel), GenerateError> {
  match model {
    Some("hunyuan_3d_3") => Ok((RouterMeshModel::Hunyuan3d3, GenerationModel::Hunyuan3d3)),
    Some("hunyuan_3d_2" | "hunyuan_3d_2_0") => Ok((RouterMeshModel::Hunyuan3d2p0, GenerationModel::Hunyuan3d2_0)),
    Some("hunyuan_3d_2_1") => Ok((RouterMeshModel::Hunyuan3d2p1, GenerationModel::Hunyuan3d2_1)),
    None => Err(GenerateError::no_model_specified()),
    Some(other) => Err(GenerateError::NotYetImplemented(format!("3D model {other} is not wired to fal yet"))),
  }
}
