use super::Kalshi;
use crate::kalshi_error::*;
use crate::utils::api_key_headers;
use crate::KalshiAuth;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use serde::{Deserialize, Serialize};

impl<'a> Kalshi {
    /// Asynchronously logs a user into the Kalshi exchange.
    ///
    /// This method sends a POST request to the Kalshi exchange's login endpoint with the user's credentials.
    /// On successful authentication, it updates the current session's token and member ID.
    ///
    /// # Arguments
    /// * `user` - A string slice representing the user's email.
    /// * `password` - A string slice representing the user's password.
    ///
    /// # Returns
    /// - `Ok(())`: Empty result indicating successful login.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request or response parsing.
    ///
    /// # Example
    /// ```
    /// kalshi_instance.login("johndoe@example.com", "example_password").await?;
    /// ```
    pub async fn login(&mut self, user: &str, password: &str) -> Result<(), KalshiError> {
        let login_url: &str = &format!("{}/login", self.base_url.to_string());

        let login_payload = LoginPayload {
            email: user.to_string(),
            password: password.to_string(),
        };

        let result: LoginResponse = self
            .client
            .post(login_url)
            .json(&login_payload)
            .send()
            .await?
            .json()
            .await?;

        self.curr_token = Some(format!("Bearer {}", result.token));
        self.member_id = Some(result.member_id);

        return Ok(());
    }

    /// Asynchronously logs a user out of the Kalshi exchange.
    ///
    /// Sends a POST request to the Kalshi exchange's logout endpoint. This method
    /// should be called to properly terminate the session initiated by `login`.
    ///
    /// # Returns
    /// - `Ok(())`: Empty result indicating successful logout.
    /// - `Err(KalshiError)`: Error in case of a failure in the HTTP request.
    ///
    /// # Examples
    /// ```
    /// kalshi_instance.logout().await?;
    /// ```
    pub async fn logout(&self) -> Result<(), KalshiError> {
        let logout_url: &str = &format!("{}/logout", self.base_url.to_string());

        self.client
            .post(logout_url)
            .header("Authorization", self.curr_token.clone().unwrap())
            .header("content-type", "application/json".to_string())
            .send()
            .await?;

        return Ok(());
    }

    /// Generates authentication headers for HTTP requests based on the current auth method.
    ///
    /// This method handles both email/password authentication (using Bearer token) and
    /// API key authentication (using KALSHI headers with signature).
    ///
    /// # Arguments
    /// * `path` - The request path for API key signing
    /// * `method` - The HTTP method for API key signing
    ///
    /// # Returns
    /// - `Ok(HeaderMap)`: Headers map with appropriate authentication headers
    /// - `Err(Box<dyn Error>)`: Error in case of missing token or signing failure
    ///
    /// # Example
    /// ```
    /// let headers = kalshi_instance.generate_auth_headers("/trade-api/ws/v2", Method::GET)?;
    /// ```
    pub fn generate_auth_headers(
        &mut self,
        path: &str,
        method: Method,
    ) -> Result<HeaderMap, KalshiError> {
        let mut headers = HeaderMap::new();

        match &mut self.auth {
            KalshiAuth::EmailPassword => {
                let curr_token = self.get_user_token().ok_or_else(|| {
                    KalshiError::UserInputError(
                        "No user token, login first using .login(..)".to_string(),
                    )
                })?;
                let header_name = HeaderName::from_static("authorization");
                let header_value = HeaderValue::from_str(&curr_token).map_err(|e| {
                    KalshiError::InternalError(format!("Invalid header value: {}", e))
                })?;
                headers.insert(header_name, header_value);
            }
            KalshiAuth::ApiKey { key_id, signer, .. } => {
                let api_key_headers =
                    api_key_headers(key_id, signer, path, method).map_err(|e| {
                        KalshiError::InternalError(format!(
                            "API key header generation failed: {}",
                            e
                        ))
                    })?;
                for (key, val) in api_key_headers {
                    let header_name = HeaderName::try_from(key).map_err(|e| {
                        KalshiError::InternalError(format!("Invalid header name '{}': {}", key, e))
                    })?;
                    let header_value = HeaderValue::from_str(val.as_str()).map_err(|e| {
                        KalshiError::InternalError(format!("Invalid header value: {}", e))
                    })?;
                    headers.insert(header_name, header_value);
                }
            }
        }

        Ok(headers)
    }
}

// used in login method
#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    member_id: String,
    token: String,
}
// used in login method
#[derive(Debug, Serialize, Deserialize)]
struct LoginPayload {
    email: String,
    password: String,
}
