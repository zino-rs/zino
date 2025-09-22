use serde::{Deserialize, Serialize};
use serde_json::{self, Value, json};
//use crate::{audio_generation::AudioGenerationError, client::video_generation};
use super::client::Client;
use crate::video_generation::{self, VideoGenerationError, VideoGenerationRequest};
use std::time::Duration;
use tokio::time::sleep;

//submit Response struct
#[derive(Debug, Deserialize, Serialize)]
pub struct SubmitResponse {
    pub task_id: Option<String>,
    pub base_resp: Option<BaseResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BaseResponse {
    pub status_code: Option<usize>,
    pub status_msg: Option<String>,
}

//check status Response struct
#[derive(Debug, Deserialize, Serialize)]
pub struct StatusResponse {
    pub task_id: String,
    pub status: Option<String>,
    pub file_id: String,
    pub video_width: u32,
    pub video_height: u32,
    pub base_resp: Option<BaseResponse>,
}

//
#[derive(Debug, Deserialize, Serialize)]
pub struct VideoGenerationResponse {
    pub file: VideoFile,
    pub base_resp: BaseResponse,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VideoFile {
    pub file_id: String,
    pub bytes: u64,
    pub created_at: u64,
    pub filename: String,
    pub purpose: String,
    pub download_url: String,
}

#[derive(Clone)]
pub struct VideoGenerationModel {
    client: Client,
    pub model: String,
}

impl VideoGenerationModel {
    pub fn new(model: &str, client: Client) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    pub fn generate_request(
        &self,
        request: VideoGenerationRequest,
    ) -> Result<serde_json::Value, VideoGenerationError> {
        // Prepare the request body based on the model type
        let mut request_body: Value = serde_json::to_value(&request)?;
        request_body["model"] = serde_json::Value::String(self.model.clone());
        Ok(request_body)
    }

    // Submit a video generation task (POST /v1/video_generation)
    pub async fn submit_task(
        &self,
        request: video_generation::VideoGenerationRequest,
    ) -> Result<SubmitResponse, VideoGenerationError> {
        // Validate body
        let body = self.generate_request(request)?;
        let resp = self
            .client
            .post("/v1/video_generation")
            .json(&body)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(VideoGenerationError::ProviderError(format!(
                "{}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        let text = resp.text().await?;
        let parsed: SubmitResponse = serde_json::from_str(&text)?;
        Ok(parsed)
    }

    // Query a video generation task (GET /v1/query/video_generation?task_id=...)
    pub async fn query_task(&self, task_id: &str) -> Result<StatusResponse, VideoGenerationError> {
        let path = format!("/v1/query/video_generation?task_id=${}", task_id);
        let resp = self.client.get(&path).send().await?;
        if !resp.status().is_success() {
            return Err(VideoGenerationError::ProviderError(format!(
                "{}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        let text = resp.text().await?;
        let value: StatusResponse = serde_json::from_str(&text)?;
        Ok(value)
    }

    // Retrieve video file URL (GET /v1/files/retrieve?file_id=...)
    pub async fn retrieve_file(
        &self,
        file_id: &str,
    ) -> Result<VideoGenerationResponse, VideoGenerationError> {
        let path = format!("/v1/files/retrieve?file_id={}", file_id);
        let resp = self.client.get(&path).send().await?;
        if !resp.status().is_success() {
            return Err(VideoGenerationError::ProviderError(format!(
                "{}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            )));
        }
        let text = resp.text().await?;
        let value: VideoGenerationResponse = serde_json::from_str(&text)?;
        Ok(value)
    }

    pub async fn wait_task(&self, task_id: String) -> Result<String, VideoGenerationError> {
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(300); // max wait time 300 seconds
        let interval = Duration::from_secs(5); // 5 seconds polling interval

        loop {
            if start_time.elapsed() > timeout {
                return Err(VideoGenerationError::ProviderError(
                    "Video generation timed out".to_string(),
                ));
            }

            let status_value = self.query_task(task_id.as_str()).await?;

            // Check for Minimax-specific response format
            let task_status = status_value.status;
            let file_id = status_value.file_id;
            match task_status.unwrap().as_str() {
                "SUCCESS" => {
                    // Ensure we have file_id when status is SUCCESS
                    return Ok(file_id);
                }
                "FAIL" => {
                    let message = status_value.base_resp.unwrap().status_msg.unwrap();
                    return Err(VideoGenerationError::ProviderError(message.to_string()));
                }
                // Processing states keep polling
                "PREPARING" | "QUEUEING" | "PROCESSING" => {
                    sleep(interval).await;
                }
                // Unknown status: keep polling but avoid tight loop
                _ => {
                    sleep(interval).await;
                }
            }
        }
    }
}

impl video_generation::VideoGenerationModel for VideoGenerationModel {
    async fn video_generation(
        &self,
        request: video_generation::VideoGenerationRequest,
    ) -> Result<video_generation::VideoGenerationResponse, VideoGenerationError> {
        // Submit the task
        let submit_resp = self.submit_task(request).await?;

        // Extract task_id from submit response
        let task_id = submit_resp.task_id.ok_or_else(|| {
            VideoGenerationError::ResponseError("Missing task_id in response".to_string())
        })?;

        // Wait for task completion by using task_id,once completed,get file_id
        let file_id = self.wait_task(task_id.clone()).await?;

        // Retrieve the video file to get download URL
        let file_info = self.retrieve_file(&file_id).await?;

        // Extract video_url and task_id from file_info
        let video_url = file_info.file.download_url;

        // Return the response with download URL
        Ok(video_generation::VideoGenerationResponse {
            video_url: Some(vec![video_url]),
            task_id: Some(task_id),
        })
    }
}
