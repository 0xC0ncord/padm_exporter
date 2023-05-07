use anyhow;
use reqwest;
use serde_json::{from_str, Value};

pub struct AuthData {
    pub access_token: String,
    pub refresh_token: String, //unused
    pub msg: String,
}
impl AuthData {
    pub fn new() -> AuthData {
        return AuthData {
            access_token: String::new(),
            refresh_token: String::new(),
            msg: String::new(),
        };
    }
    pub fn is_empty(&self) -> bool {
        return self.access_token == "" ||
            self.refresh_token == "" ||
            self.msg == "";
    }
}

/*
* Client for interacting with PADM devices
*/
pub struct PADMClient {
    client: reqwest::Client,
    pub host: String,
    pub scheme: String,
    username: String,
    password: String,
    auth_data: AuthData,
}
impl PADMClient {
    /*
    * Create a new PADMClient
    */
    pub async fn new(host: &str, scheme: &str, tls_insecure: bool, username: &str, password: &str) -> PADMClient {
        let mut client_builder = reqwest::Client::builder();
        // Disable SSL verification if asked
        if tls_insecure {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        // Get a new reqwest client
        let client = client_builder.build().unwrap();

        return PADMClient {
            client,
            host: String::from(host),
            scheme: String::from(scheme),
            username: username.to_string(),
            password: password.to_string(),
            auth_data: AuthData::new(),
        }
    }

    /*
    * Log into the device and retrieve authentication data
    */
    async fn authenticate(&mut self) -> anyhow::Result<()> {
        let request_url = format!("https://{}/api/oauth/token?grant_type=password", self.host);
        let params = [("username", &self.username), ("password", &self.password)];

        let response = self.client.post(&request_url)
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_response: Value = from_str(&response)?;

        fn extract(json: &Value, field: &str) -> String {
            return json[field].to_string()
                .trim_matches('"')
                .to_string();
        }

        self.auth_data = AuthData {
            access_token: extract(&json_response, "access_token"),
            refresh_token: extract(&json_response, "refresh_token"),
            msg: extract(&json_response, "msg"),
        };
        Ok(())
    }

    async fn raw_get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.client.get(url)
            .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", &self.auth_data.access_token))
            .send()
            .await
    }

    /*
    * Do an authenticated GET request
    */
    pub async fn do_get(&mut self, path: &str) -> anyhow::Result<reqwest::Response> {
        let url = format!("{}://{}{}", self.scheme, self.host, path);

        // Authenticate if never authenticated before
        if self.auth_data.is_empty() {
            self.authenticate().await?;
        }

        let response = self.raw_get(&url).await;
        return match response {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(code) = e.status() {
                    // Authenticate again if needed
                    if code == reqwest::StatusCode::FORBIDDEN {
                        self.authenticate().await?;
                        return Ok(self.raw_get(&url).await?);
                    }
                }
                // Otherwise just return the error
                return Err(anyhow::Error::new(e));
            }
        }
    }
}
