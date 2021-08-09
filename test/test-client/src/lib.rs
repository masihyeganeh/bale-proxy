tonic::include_proto!("bale.auth.v1");

use bale::BaleClient;
use grpc_web_client::{Client, Encoding};

#[tokio::test]
async fn hello_world() {
    let base_client =
        Client::new_with_encoding("https://next-api.bale.ai".to_string(), Encoding::Base64);

    let mut client = auth_client::AuthClient::new(base_client);

    let mut request = tonic::Request::new(StartPhoneAuthRequest {
        phone_number: 13652972128,
        client_version: 4,
        api_key: "C28D46DC4C3A7A26564BFCC48B929086A95C93C98E789A19847BEE8627DE4E7D".to_string(),
        user_agent: "Firefox, macOS".to_string(),
        user_agent_string: "Firefox, macOS".to_string(),
    });

    request.metadata_mut().insert("auth-jwt", "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE2MzM1OTExODI1OTQsImlhdCI6MTYyODQwNzE4MjU5NCwiaXNzIjoiaHR0cHM6Ly93ZWIuYmFsZS5haSIsInBheWxvYWQiOnsiYXBwX2lkIjo0LCJhdXRoX2lkIjoiMTgxOTc2OTA4MTk4MzU4Njk3MCIsImF1dGhfc2lkIjo1ODE0NzYwMTYsInNlcnZpY2UiOiJ3ZWJfbGl0ZSIsInVzZXJfaWQiOjcwMzcyOTQ0N319.bliASxltkfu77u8XtFLq-4Vhk2RTLXPp0eYJDeL5f9Ag8YPz7QFTgJZBlwQiUCfh-50HiCQpOC27a6Z0giFqeGwN4H4boI3BR8dVc_9kjk-hoMnMogcWHmhISVq6oIQwAzeg_SWFrtA5Au0WTArLS6emQzoLyopUyo7kz_n3ONhIrgxTY5U-KY3dEnipLyTzXL91ctLi_2sX8PNq5mOS1QWjTpas05ZMCqCfRq4DkzOCXM_4CDyaR_Dm85LlhxO2Kk4gHnRAJBdOmpa1XJYUoSo_wmMAOfz1QM9bOmYhTUYgFBHOmuClYdGEtKBOfv6DbTnZs4C0EIm5-HP6T13ubeTXUJHHTclDKU1SZydPqoU_pX29zGZvPexZelfEU5VsNVIxvXz9KId2nMWJ9G_lzgFXSJmjYFH_7_ygKc29BA0eC6SkouJ22VexCUO5llSkuLPds-FfVsvbzzhArvcRdMZMoa8DZbHWZ0eggcbB8c6wtPP4-DlfgS7ZwtUQsPTVnd5nKUn5_wOfMeF2KNW4VWycz8dJ043pZzOiyuGgok3ddqhq9oMxuVc7MtYop58UpoMttUiHkCGlpFeIN7TvXqbHco0BXMEtEMF0eCp4RBJ00bMa0Wjzwlz2OxKXgz8VpVLuSmGpiPpckS_x2Wa93eR4Tk67G_XbIQFPn-0cuTg".parse().unwrap());
    let response = client.start_phone_auth(request).await.unwrap().into_inner();
    eprintln!("{}", response.login_hash)
}

#[tokio::test]
async fn folan() {
    let mut bale = BaleClient::new();
    let registered = bale.login(4915207829731).await;
    let login_code = "1234";
    bale.validate_code(login_code).await;
}
