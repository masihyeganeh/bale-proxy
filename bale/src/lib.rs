tonic::include_proto!("bale.v1");
tonic::include_proto!("bale.auth.v1");
tonic::include_proto!("bale.messaging.v2");
tonic::include_proto!("bale.maviz.v1");

use grpc_web_client::{Client, Encoding};
use std::collections::HashMap;

const API_KEY: &str = "C28D46DC4C3A7A26564BFCC48B929086A95C93C98E789A19847BEE8627DE4E7D";

pub struct BaleClient {
    client: Client,
    login_status: LoginStatus,
    phone_number: u64,
}

#[derive(Clone)]
pub enum LoginStatus {
    WaitingForNumber,
    WaitingForValidationCode(String),
    LoggedIn(String),
    NotRegistered,
    Expired,
    Error(String),
}

impl BaleClient {
    pub fn new(phone_number: u64) -> Self {
        BaleClient {
            client: Client::new_with_encoding(
                "https://next-api.bale.ai".to_string(),
                Encoding::Base64,
            ),
            login_status: LoginStatus::WaitingForNumber,
            phone_number,
        }
    }

    pub async fn login(&mut self) -> LoginStatus {
        let mut client = auth_client::AuthClient::new(self.client.clone());

        let request = tonic::Request::new(StartPhoneAuthRequest {
            phone_number: self.phone_number,
            client_version: 4,
            api_key: API_KEY.to_string(),
            user_agent: "Firefox, macOS".to_string(),
            user_agent_string: "Firefox, macOS".to_string(),
        });

        let response = match client.start_phone_auth(request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                panic!("{} : {}", status.code(), status.message())
            }
        };

        self.login_status = LoginStatus::WaitingForValidationCode(response.login_hash);

        if response.registered != 1 {
            return LoginStatus::NotRegistered;
        }

        let _conf = self.fetch_configs().await;
        self.login_status.clone()
    }

    pub async fn login_with(&mut self, jwt: String) -> LoginStatus {
        self.login_status = LoginStatus::LoggedIn(jwt);
        if let Some(_configs) = self.fetch_configs().await {
            //
        } else {
            self.login_status = LoginStatus::Expired;
        }
        self.login_status.clone()
    }

    async fn fetch_configs(&mut self) -> Option<HashMap<String, String>> {
        let mut client = configs_client::ConfigsClient::new(self.client.clone());

        let mut request = tonic::Request::new(GetParametersRequest {});

        let jwt = if let LoginStatus::LoggedIn(jwt) = &self.login_status {
            jwt
        } else {
            return None;
        };

        request
            .metadata_mut()
            .insert("auth-jwt", jwt.parse().unwrap());

        let mut response = match client.get_parameters(request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                println!("{} : {}", status.code(), status.message());
                return None;
            }
        };

        if let Ok(Some(reply)) = response.message().await {
            Some(
                reply
                    .configs
                    .into_iter()
                    .fold(HashMap::new(), |mut configs, config| {
                        configs.insert(config.key, config.value);
                        configs
                    }),
            )
        } else {
            None
        }
    }

    pub async fn validate_code(&mut self, login_code: &str) -> Option<Profile> {
        let mut client = auth_client::AuthClient::new(self.client.clone());

        let login_hash =
            if let LoginStatus::WaitingForValidationCode(login_hash) = &self.login_status {
                login_hash
            } else {
                return None;
            };

        let request = tonic::Request::new(ValidateCodeRequest {
            login_hash: login_hash.to_string(),
            login_code: login_code.to_string(),
            validate_code_request_sub_request: Some(ValidateCodeRequestSubRequest { unknown: 1 }),
        });

        let response = match client.validate_code(request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                panic!("{} : {}", status.code(), status.message())
            }
        };

        self.login_status = LoginStatus::LoggedIn(response.auth.clone().unwrap().jwt);
        eprintln!("{:#?}", response);

        Some(response.profile.clone().unwrap())
    }

    pub async fn send_message(&self, user_id: u32, message: String) {
        let mut client = messaging_client::MessagingClient::new(self.client.clone());

        let jwt = if let LoginStatus::LoggedIn(jwt) = &self.login_status {
            jwt
        } else {
            return;
        };

        let mut request = tonic::Request::new(SendMessageRequest {
            peer: Some(MessagingPeer {
                unknown: 1,
                user_id,
            }),
            rid: rand::random(),
            message: Some(MessagingMessage {
                text_message: Some(MessagingTextMessage { text: message }),
            }),
        });

        request
            .metadata_mut()
            .insert("auth-jwt", jwt.parse().unwrap());

        let response = match client.send_message(request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                panic!("{} : {}", status.code(), status.message())
            }
        };

        eprintln!("{:#?}", response)
    }

    pub async fn subscribe_to_updates(&self) {
        let mut client = maviz_stream_client::MavizStreamClient::new(self.client.clone());

        let jwt = if let LoginStatus::LoggedIn(jwt) = &self.login_status {
            jwt
        } else {
            return;
        };

        let mut request = tonic::Request::new(SubscribeToUpdatesRequest {});

        request
            .metadata_mut()
            .insert("auth-jwt", jwt.parse().unwrap());

        let mut response = match client.subscribe_to_updates(request).await {
            Ok(response) => response.into_inner(),
            Err(status) => {
                panic!("{} : {}", status.code(), status.message())
            }
        };

        while let message = response.message().await {
            eprintln!("{:#?}", message)
        }
    }
}
