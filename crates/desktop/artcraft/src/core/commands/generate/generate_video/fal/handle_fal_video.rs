use artcraft_router::api::image_list_ref::ImageListRef;
use artcraft_router::api::image_ref::ImageRef;
use artcraft_router::api::router_provider::RouterProvider;
use artcraft_router::client::request_mismatch_mitigation_strategy::RequestMismatchMitigationStrategy;
use artcraft_router::client::router_client::RouterClient;
use artcraft_router::client::router_fal_client::RouterFalClient;
use artcraft_router::generate::generate_video::generate_video_request_builder::GenerateVideoRequestBuilder;
use artcraft_router::generate::generate_video::video_generation_draft_or_request::VideoGenerationDraftOrRequest;
use enums::common::generation_provider::GenerationProvider;
use enums::tauri::tasks::task_type::TaskType;
use log::{info, warn};
use tokens::tokens::media_files::MediaFileToken;

use crate::core::api_adapters::models::video::tauri_video_model_to_generation_model::tauri_video_model_to_generation_model;
use crate::core::api_adapters::models::video::tauri_video_model_to_router_model::tauri_video_model_to_router_model;
use crate::core::commands::enqueue::generate_error::{GenerateError, MissingCredentialsReason};
use crate::core::commands::enqueue::task_enqueue_success::TaskEnqueueSuccess;
use crate::core::commands::generate::generate_image::providers::artcraft_router::utils::map_media_files_to_urls::map_media_file_tokens_to_cdn_urls;
use crate::core::commands::generate::generate_video::request::TauriGenerateVideoRequest;
use crate::core::providers::credentials::payload::provider_credential_payload::ProviderCredentialPayload;
use crate::core::providers::credentials::provider_credential_key::ProviderCredentialKey;
use crate::core::providers::credentials::provider_credential_loading_cache::ProviderCredentialLoadingCache;
use crate::core::state::app_env_configs::app_env_configs::AppEnvConfigs;

/// Send a video generation straight to fal with the user's own API key.
/// The finished video is picked up by the fal task polling thread.
pub async fn handle_fal_video(
  request: &TauriGenerateVideoRequest,
  app_env_configs: &AppEnvConfigs,
  credential_cache: &ProviderCredentialLoadingCache,
) -> Result<TaskEnqueueSuccess, GenerateError> {
  let tauri_model = request.model.ok_or(GenerateError::no_model_specified())?;
  let api_key = load_fal_api_key(credential_cache)?;
  let api_host = &app_env_configs.storyteller_host;

  // fal needs public URLs, so resolve the uploaded frames to their CDN links.
  let start_frame = resolve_one(request.start_frame_image_media_token.as_ref(), api_host).await?;
  let end_frame = resolve_one(request.end_frame_image_media_token.as_ref(), api_host).await?;
  let reference_images = match &request.reference_image_media_tokens {
    Some(tokens) if !tokens.is_empty() => {
      Some(ImageListRef::Urls(map_media_file_tokens_to_cdn_urls(tokens, api_host).await?))
    }
    _ => None,
  };

  let router_request = GenerateVideoRequestBuilder {
    model: tauri_video_model_to_router_model(tauri_model),
    provider: RouterProvider::Fal,
    prompt: request.prompt.clone(),
    negative_prompt: None,
    start_frame,
    end_frame,
    reference_images,
    reference_videos: None,
    reference_audio: None,
    reference_character_tokens: None,
    resolution: request.resolution,
    aspect_ratio: request.aspect_ratio,
    bitrate: None,
    duration_seconds: request.duration_seconds,
    video_batch_count: request.video_batch_count,
    generate_audio: request.generate_audio,
    total_reference_video_input_seconds: None,
    request_mismatch_mitigation_strategy: RequestMismatchMitigationStrategy::PayMoreUpgrade,
    idempotency_token: None,
  };

  let request_to_send = match router_request.build2()? {
    VideoGenerationDraftOrRequest::Request(r) => r,
    VideoGenerationDraftOrRequest::Draft(_) => {
      return Err(GenerateError::NotYetImplemented("fal video should not produce a draft request".to_string()));
    }
  };

  let client = RouterClient::Fal(RouterFalClient::new_polling_only_from_raw_key(&api_key));

  info!("Executing fal video generation: model={:?}", tauri_model);

  let response = request_to_send.send_request(&client).await.map_err(|err| {
    warn!("fal video generation failed: {:?}", err);
    GenerateError::from(err)
  })?;

  let fal_payload = response.get_fal_payload().ok_or(GenerateError::ResponseHadNoJobTokens)?;
  let provider_job_id = fal_payload.request_id
    .or(fal_payload.gateway_request_id)
    .ok_or(GenerateError::ResponseHadNoJobTokens)?;

  Ok(TaskEnqueueSuccess {
    task_type: TaskType::VideoGeneration,
    model: Some(tauri_video_model_to_generation_model(tauri_model)),
    provider: GenerationProvider::Fal,
    provider_job_id: Some(provider_job_id),
    maybe_queue_status_url: fal_payload.maybe_status_url,
    maybe_queue_response_url: fal_payload.maybe_response_url,
    maybe_prompt_token: None,
  })
}

pub(crate) fn load_fal_api_key(
  credential_cache: &ProviderCredentialLoadingCache,
) -> Result<String, GenerateError> {
  let payload = credential_cache.get_credentials(ProviderCredentialKey::FalApiKey)
    .map_err(|err| GenerateError::AnyhowError(anyhow::anyhow!("Failed to load fal key: {:?}", err)))?
    .ok_or(GenerateError::MissingCredentials(MissingCredentialsReason::NeedsFalApiKey))?;
  match payload {
    ProviderCredentialPayload::ApiKey(data) => Ok(data.as_str().to_string()),
    _ => Err(GenerateError::MissingCredentials(MissingCredentialsReason::NeedsFalApiKey)),
  }
}

async fn resolve_one(
  token: Option<&MediaFileToken>,
  api_host: &artcraft_client::utils::api_host::ApiHost,
) -> Result<Option<ImageRef>, GenerateError> {
  let Some(token) = token else { return Ok(None) };
  let urls = map_media_file_tokens_to_cdn_urls(&[token.clone()], api_host).await?;
  Ok(urls.into_iter().next().map(ImageRef::Url))
}
