tonic::include_proto!("bale.auth.v1");
tonic::include_proto!("bale.messaging.v2");
tonic::include_proto!("bale.maviz.v1");

use crate::LoginStatus::{LoggedIn, WaitingForNumber, WaitingForValidationCode};
use grpc_web_client::{Client, Encoding};

const API_KEY: &str = "C28D46DC4C3A7A26564BFCC48B929086A95C93C98E789A19847BEE8627DE4E7D";

pub struct BaleClient {
    client: Client,
    login_status: LoginStatus,
}

enum LoginStatus {
    WaitingForNumber,
    WaitingForValidationCode(String),
    LoggedIn(String, Profile),
}

impl BaleClient {
    pub fn new() -> Self {
        BaleClient {
            client: Client::new_with_encoding(
                "https://next-api.bale.ai".to_string(),
                Encoding::Base64,
            ),
            login_status: WaitingForNumber,
        }
    }

    pub async fn login(&mut self, phone_number: u64) -> bool {
        let mut client = auth_client::AuthClient::new(self.client.clone());

        let request = tonic::Request::new(StartPhoneAuthRequest {
            phone_number,
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

        self.login_status = WaitingForValidationCode(response.login_hash);
        response.registered == 1
    }

    pub async fn validate_code(&mut self, login_code: &str) {
        let mut client = auth_client::AuthClient::new(self.client.clone());

        let login_hash = if let WaitingForValidationCode(login_hash) = &self.login_status {
            login_hash
        } else {
            return;
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

        self.login_status = LoggedIn(
            response.auth.clone().unwrap().jwt,
            response.profile.clone().unwrap(),
        );
        eprintln!("{:#?}", response)
    }

    pub async fn send_message(&self, user_id: u32, message: String) {
        let mut client = messaging_client::MessagingClient::new(self.client.clone());

        let (jwt, profile) = if let LoggedIn(jwt, profile) = &self.login_status {
            (jwt, profile)
        } else {
            return;
        };

        let mut request = tonic::Request::new(SendMessageRequest {
            peer: Some(Peer {
                unknown: 1,
                user_id,
            }),
            rid: rand::random(),
            message: Some(Message {
                text_message: Some(TextMessage { text: message }),
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

        let (jwt, profile) = if let LoggedIn(jwt, profile) = &self.login_status {
            (jwt, profile)
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
