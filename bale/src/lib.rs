tonic::include_proto!("bale.auth.v1");

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
    LoggedIn(String),
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

        let mut request = tonic::Request::new(StartPhoneAuthRequest {
            phone_number,
            client_version: 4,
            api_key: API_KEY.to_string(),
            user_agent: "Firefox, macOS".to_string(),
            user_agent_string: "Firefox, macOS".to_string(),
        });

        // request.metadata_mut().insert("auth-jwt", "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2MzM1OTExODI1OTQsImlhdCI6MTYyODQwNzE4MjU5NCwiaXNzIjoiaHR0cHM6Ly93ZWIuYmFsZS5haSIsInBheWxvYWQiOnsiYXBwX2lkIjo0LCJhdXRoX2lkIjoiMTgxOTc2OTA4MTk4MzU4Njk3MCIsImF1dGhfc2lkIjo1ODE0NzYwMTYsInNlcnZpY2UiOiJ3ZWJfbGl0ZSIsInVzZXJfaWQiOjcwMzcyOTQ0N319.bliASxltkfu77u8XtFLq-4Vhk2RTLXPp0eYJDeL5f9Ag8YPz7QFTgJZBlwQiUCfh-50HiCQpOC27a6Z0giFqeGwN4H4boI3BR8dVc_9kjk-hoMnMogcWHmhISVq6oIQwAzeg_SWFrtA5Au0WTArLS6emQzoLyopUyo7kz_n3ONhIrgxTY5U-KY3dEnipLyTzXL91ctLi_2sX8PNq5mOS1QWjTpas05ZMCqCfRq4DkzOCXM_4CDyaR_Dm85LlhxO2Kk4gHnRAJBdOmpa1XJYUoSo_wmMAOfz1QM9bOmYhTUYgFBHOmuClYdGEtKBOfv6DbTnZs4C0EIm5-HP6T13ubeTXUJHHTclDKU1SZydPqoU_pX29zGZvPexZelfEU5VsNVIxvXz9KId2nMWJ9G_lzgFXSJmjYFH_7_ygKc29BA0eC6SkouJ22VexCUO5llSkuLPds-FfVsvbzzhArvcRdMZMoa8DZbHWZ0eggcbB8c6wtPP4-DlfgS7ZwtUQsPTVnd5nKUn5_wOfMeF2KNW4VWycz8dJ043pZzOiyuGgok3ddqhq9oMxuVc7MtYop58UpoMttUiHkCGlpFeIN7TvXqbHco0BXMEtEMF0eCp4RBJ00bMa0Wjzwlz2OxKXgz8VpVLuSmGpiPpckS_x2Wa93eR4Tk67G_XbIQFPn-0cuTg".parse().unwrap());
        let response = client.start_phone_auth(request).await.unwrap().into_inner();
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

        let mut request = tonic::Request::new(ValidateCodeRequest {
            login_hash: login_hash.to_string(),
            login_code: login_code.to_string(),
            validate_code_request_sub_request: Some(ValidateCodeRequestSubRequest { unknown: 1 }),
        });

        // request.metadata_mut().insert("auth-jwt", "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2MzM1OTExODI1OTQsImlhdCI6MTYyODQwNzE4MjU5NCwiaXNzIjoiaHR0cHM6Ly93ZWIuYmFsZS5haSIsInBheWxvYWQiOnsiYXBwX2lkIjo0LCJhdXRoX2lkIjoiMTgxOTc2OTA4MTk4MzU4Njk3MCIsImF1dGhfc2lkIjo1ODE0NzYwMTYsInNlcnZpY2UiOiJ3ZWJfbGl0ZSIsInVzZXJfaWQiOjcwMzcyOTQ0N319.bliASxltkfu77u8XtFLq-4Vhk2RTLXPp0eYJDeL5f9Ag8YPz7QFTgJZBlwQiUCfh-50HiCQpOC27a6Z0giFqeGwN4H4boI3BR8dVc_9kjk-hoMnMogcWHmhISVq6oIQwAzeg_SWFrtA5Au0WTArLS6emQzoLyopUyo7kz_n3ONhIrgxTY5U-KY3dEnipLyTzXL91ctLi_2sX8PNq5mOS1QWjTpas05ZMCqCfRq4DkzOCXM_4CDyaR_Dm85LlhxO2Kk4gHnRAJBdOmpa1XJYUoSo_wmMAOfz1QM9bOmYhTUYgFBHOmuClYdGEtKBOfv6DbTnZs4C0EIm5-HP6T13ubeTXUJHHTclDKU1SZydPqoU_pX29zGZvPexZelfEU5VsNVIxvXz9KId2nMWJ9G_lzgFXSJmjYFH_7_ygKc29BA0eC6SkouJ22VexCUO5llSkuLPds-FfVsvbzzhArvcRdMZMoa8DZbHWZ0eggcbB8c6wtPP4-DlfgS7ZwtUQsPTVnd5nKUn5_wOfMeF2KNW4VWycz8dJ043pZzOiyuGgok3ddqhq9oMxuVc7MtYop58UpoMttUiHkCGlpFeIN7TvXqbHco0BXMEtEMF0eCp4RBJ00bMa0Wjzwlz2OxKXgz8VpVLuSmGpiPpckS_x2Wa93eR4Tk67G_XbIQFPn-0cuTg".parse().unwrap());
        let response = client.validate_code(request).await.unwrap().into_inner();
        self.login_status = LoggedIn(response.auth.clone().unwrap().jwt);
        eprintln!("{:#?}", response)
    }
}
