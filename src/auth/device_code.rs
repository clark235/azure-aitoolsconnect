use super::{AuthProvider, Credentials};
use crate::config::Cloud;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::devicecode::StandardDeviceAuthorizationResponse;
use oauth2::{
    AuthUrl, ClientId, DeviceAuthorizationUrl, DeviceCodeErrorResponseType, RequestTokenError,
    Scope, TokenResponse, TokenUrl,
};
use std::time::Duration;
use tokio::time::sleep;

/// Azure CLI's well-known public client ID
const AZURE_CLI_CLIENT_ID: &str = "04b07795-8ddb-461a-bbee-02f9e1bf7b46";

/// Device Code Flow authentication provider
/// Displays a code for the user to enter at a Microsoft login page
pub struct DeviceCodeAuth {
    tenant_id: String,
    client_id: String,
    scope: String,
    cloud: Cloud,
}

impl DeviceCodeAuth {
    pub fn new(tenant_id: String, client_id: Option<String>, cloud: &Cloud) -> Result<Self> {
        let client_id = client_id.unwrap_or_else(|| AZURE_CLI_CLIENT_ID.to_string());

        let scope = match cloud {
            Cloud::Global => "https://cognitiveservices.azure.com/.default",
            Cloud::China => "https://cognitiveservices.azure.cn/.default",
        };

        Ok(Self {
            tenant_id,
            client_id,
            scope: scope.to_string(),
            cloud: *cloud,
        })
    }

    /// Fetch token using device code flow
    async fn fetch_token(&self) -> Result<String> {
        let login_endpoint = self.cloud.login_endpoint();

        // Build OAuth2 client
        let device_auth_url = DeviceAuthorizationUrl::new(format!(
            "{}/{}/oauth2/v2.0/devicecode",
            login_endpoint, self.tenant_id
        ))
        .map_err(|e| AppError::DeviceCodeAuthFailed(format!("Invalid device auth URL: {}", e)))?;

        let token_url = TokenUrl::new(format!(
            "{}/{}/oauth2/v2.0/token",
            login_endpoint, self.tenant_id
        ))
        .map_err(|e| AppError::DeviceCodeAuthFailed(format!("Invalid token URL: {}", e)))?;

        let auth_url = AuthUrl::new(format!(
            "{}/{}/oauth2/v2.0/authorize",
            login_endpoint, self.tenant_id
        ))
        .map_err(|e| AppError::DeviceCodeAuthFailed(format!("Invalid auth URL: {}", e)))?;

        let client = BasicClient::new(
            ClientId::new(self.client_id.clone()),
            None,
            auth_url,
            Some(token_url),
        )
        .set_device_authorization_url(device_auth_url);

        // Request device code
        let details: StandardDeviceAuthorizationResponse = client
            .exchange_device_code()
            .map_err(|e| {
                AppError::DeviceCodeAuthFailed(format!("Failed to initiate device code flow: {}", e))
            })?
            .add_scope(Scope::new(self.scope.clone()))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| {
                AppError::DeviceCodeAuthFailed(format!("Device code request failed: {}", e))
            })?;

        // Display instructions to user
        self.display_instructions(&details);

        // Poll for token
        let token = self.poll_for_token(&client, &details).await?;

        Ok(token.access_token().secret().clone())
    }

    /// Display authentication instructions to the user
    fn display_instructions(&self, details: &StandardDeviceAuthorizationResponse) {
        println!("\n{}", "=".repeat(70));
        println!("  Azure Authentication Required");
        println!("{}", "=".repeat(70));
        println!();
        println!("  Please visit:  {}", details.verification_uri().as_str());
        println!();
        println!("  And enter code:  {}", details.user_code().secret());
        println!();
        println!("{}", "=".repeat(70));
        println!();
        println!("Waiting for authentication...");
        println!();
    }

    /// Poll the token endpoint until the user completes authentication
    async fn poll_for_token(
        &self,
        client: &BasicClient,
        details: &StandardDeviceAuthorizationResponse,
    ) -> Result<BasicTokenResponse> {
        let interval = details.interval();
        let timeout = Duration::from_secs(15 * 60); // 15 minutes
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(AppError::DeviceCodeAuthFailed(
                    "Authentication timeout (15 minutes). Please try again.".to_string(),
                ));
            }

            sleep(interval).await;

            match client
                .exchange_device_access_token(details)
                .request_async(
                    oauth2::reqwest::async_http_client,
                    tokio::time::sleep,
                    None,
                )
                .await
            {
                Ok(token) => {
                    println!("✓ Authentication successful!\n");
                    return Ok(token);
                }
                Err(RequestTokenError::ServerResponse(err)) => {
                    match err.error() {
                        DeviceCodeErrorResponseType::AuthorizationPending => {
                            // Still waiting for user - continue polling
                            continue;
                        }
                        DeviceCodeErrorResponseType::SlowDown => {
                            // Server requested slower polling - add extra delay
                            sleep(interval).await;
                            continue;
                        }
                        DeviceCodeErrorResponseType::ExpiredToken => {
                            return Err(AppError::DeviceCodeAuthFailed(
                                "Device code expired. Please try again.".to_string(),
                            ));
                        }
                        DeviceCodeErrorResponseType::AccessDenied => {
                            return Err(AppError::DeviceCodeAuthFailed(
                                "User declined authorization".to_string(),
                            ));
                        }
                        _ => {
                            return Err(AppError::DeviceCodeAuthFailed(format!(
                                "Server error: {:?}",
                                err
                            )));
                        }
                    }
                }
                Err(RequestTokenError::Request(e)) => {
                    return Err(AppError::DeviceCodeAuthFailed(format!(
                        "Network error during token request: {}",
                        e
                    )));
                }
                Err(e) => {
                    return Err(AppError::DeviceCodeAuthFailed(format!(
                        "Token request failed: {}",
                        e
                    )));
                }
            }
        }
    }
}

#[async_trait]
impl AuthProvider for DeviceCodeAuth {
    async fn get_credentials(&self) -> Result<Credentials> {
        let token = self.fetch_token().await?;
        Ok(Credentials::BearerToken(token))
    }

    fn method_name(&self) -> &'static str {
        "Device Code Flow"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_code_auth_creation() {
        let auth = DeviceCodeAuth::new(
            "tenant-id".to_string(),
            None,
            &Cloud::Global,
        );
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.client_id, AZURE_CLI_CLIENT_ID);
    }

    #[test]
    fn test_device_code_auth_custom_client_id() {
        let custom_id = "custom-client-id".to_string();
        let auth = DeviceCodeAuth::new(
            "tenant-id".to_string(),
            Some(custom_id.clone()),
            &Cloud::Global,
        );
        assert!(auth.is_ok());
        let auth = auth.unwrap();
        assert_eq!(auth.client_id, custom_id);
    }

    #[test]
    fn test_china_cloud_scope() {
        let auth = DeviceCodeAuth::new(
            "tenant-id".to_string(),
            None,
            &Cloud::China,
        )
        .unwrap();
        assert!(auth.scope.contains("cognitiveservices.azure.cn"));
    }
}
