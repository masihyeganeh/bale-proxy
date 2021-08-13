use async_std::net::SocketAddrV4;
use bale::{BaleClient, LoginStatus};
use std::env;
use std::io::{stdin, stdout, Write};
mod socket;
use log::trace;
use socket::Socket;

mod error;
mod simple_server;
use simple_server::get_from_web;

async fn temporary_echo_function(input_bytes: Vec<u8>) -> Vec<u8> {
    trace!("{:?}", input_bytes);
    input_bytes
}

#[tokio::main]
async fn main() {
    let run_params = get_run_params().await;

    eprintln!("Run params : {:#?}", run_params);

    let phone_number = run_params.phone_number.expect("bad phone number");

    let mut bale = BaleClient::new(phone_number);

    let login_status = if let Some(jwt) = run_params.jwt {
        let mut res = bale.login_with(jwt.to_string()).await;
        if matches!(res, LoginStatus::Expired) {
            res = bale.login().await;
        }
        res
    } else {
        bale.login().await
    };

    let user_id = match login_status {
        LoginStatus::LoggedIn(_jwt, user_id) => {
            // Successful login
            user_id
        }
        LoginStatus::WaitingForValidationCode(_) => {
            let login_code = get_input(
                format!("Please enter login code sent by sms to {}:", phone_number),
                !run_params.running_from_shadowsocks,
            )
            .await;
            if let Some(user_data) = bale.validate_code(&login_code).await {
                // Successful login
                user_data.user_id
            } else {
                panic!("wrong validation code")
            }
        }
        LoginStatus::NotRegistered => panic!("phone number is not registered"),
        LoginStatus::Error(err) => panic!("{}", err),
        _ => panic!("unknown error happened"),
    };

    if let OperationMode::Client(server_user_id) = run_params.mode {
        // TODO: Handshake
        bale.send_message(server_user_id, "Handshaking".to_string())
            .await;
    } else {
        eprintln!("Set {} as server id in clients", user_id);
    }

    Socket::new(temporary_echo_function)
        .connect(
            run_params.mode,
            SocketAddrV4::new(
                run_params.local_host.unwrap().parse().unwrap(),
                run_params.local_port.unwrap().parse().unwrap(),
            ),
            SocketAddrV4::new(
                run_params.remote_host.unwrap().parse().unwrap(),
                run_params.remote_port.unwrap().parse().unwrap(),
            ),
        )
        .await
        .unwrap();

    bale.send_message(932014429, "Hi".to_string()).await;

    bale.subscribe_to_updates().await;
}

async fn get_input(message: String, has_terminal_access: bool) -> String {
    if has_terminal_access {
        get_input_from_terminal(message)
    } else {
        get_input_from_http(message).await
    }
}

async fn get_input_from_http(message: String) -> String {
    get_from_web(message).await.unwrap()
}

fn get_input_from_terminal(message: String) -> String {
    let mut user_input = String::new();
    print!("{} ", message);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut user_input)
        .expect("Did not enter a correct string");
    if let Some('\n') = user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r') = user_input.chars().next_back() {
        user_input.pop();
    }
    user_input.trim().to_string()
}

#[derive(Debug)]
struct RunParams {
    running_from_shadowsocks: bool,
    phone_number: Option<u64>,
    jwt: Option<String>,
    remote_host: Option<String>,
    remote_port: Option<String>,
    local_host: Option<String>,
    local_port: Option<String>,
    opts: Option<String>,
    mode: OperationMode,
}

#[derive(Debug)]
pub(crate) enum OperationMode {
    Server,
    Client(u32),
}

async fn get_run_params() -> RunParams {
    let remote_host: Option<String> = env::var("SS_REMOTE_HOST").ok();
    let remote_port: Option<String> = env::var("SS_REMOTE_PORT").ok();
    let local_host: Option<String> = env::var("SS_LOCAL_HOST").ok();
    let local_port: Option<String> = env::var("SS_LOCAL_PORT").ok();
    let mut opts: Option<String> = env::var("SS_PLUGIN_OPTIONS").ok();
    let mut mode: OperationMode = OperationMode::Server;

    let mut phone_number: Option<u64> = None;
    let mut jwt: Option<String> = None;
    opts = opts.map(|opts| {
        opts.split(';')
            .map(|opt| opt.trim())
            .filter(|opt| {
                if opt.to_lowercase().starts_with("phone_number=") {
                    phone_number = opt["phone_number=".len()..].parse().ok();
                    return false;
                } else if opt.to_lowercase().starts_with("jwt=") {
                    jwt = Some((&opt["jwt=".len()..]).to_string());
                    return false;
                } else if opt.to_lowercase().starts_with("client=") {
                    mode = OperationMode::Client(opt["client=".len()..].parse().unwrap());
                    return false;
                } else if opt.to_lowercase() == "server" {
                    mode = OperationMode::Server;
                    return false;
                }
                true
            })
            .collect::<Vec<&str>>()
            .join(";")
    });

    let running_from_shadowsocks = matches!(
        (&remote_host, &remote_port, &local_host, &local_port),
        (Some(_), Some(_), Some(_), Some(_))
    );

    if phone_number == None {
        let phone_number_str = get_input(
            "Please enter mobile number (can be from receive-smss.com):".to_string(),
            !running_from_shadowsocks,
        )
        .await;
        phone_number = phone_number_str.parse().ok()
    }

    RunParams {
        running_from_shadowsocks,
        phone_number,
        jwt,
        remote_host,
        remote_port,
        local_host,
        local_port,
        opts,
        mode,
    }
}
