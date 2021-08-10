use bale::{BaleClient, LoginStatus};
use std::io::{stdin, stdout, Write};

mod error;
mod simple_server;
use simple_server::get_from_web;

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

    match login_status {
        LoginStatus::LoggedIn(_) => {
            // Successful login
        }
        LoginStatus::WaitingForValidationCode(_) => {
            let login_code = get_input(
                format!("Please enter login code sent by sms to {}:", phone_number),
                !run_params.running_from_shadowsocks,
            )
            .await;
            if bale.validate_code(&login_code).await.is_none() {
                panic!("wrong validation code")
            }
            // Successful login
        }
        LoginStatus::NotRegistered => panic!("phone number is not registered"),
        LoginStatus::Error(err) => panic!("{}", err),
        _ => panic!("unknown error happened"),
    }

    // let recipient_user_id = get_input(
    //     "Please enter recipient user id:",
    //     !run_params.running_from_shadowsocks,
    // )
    // .await;
    // let message = get_input(
    //     "Please enter your message:",
    //     !run_params.running_from_shadowsocks,
    // )
    // .await;
    // bale.send_message(recipient_user_id.parse().unwrap(), message)
    //     .await;
    bale.send_message(932014429, "Hi".to_string()).await;

    if let OperationMode::Client(server_user_id) = run_params.mode {
        // TODO: Handshake
        bale.send_message(server_user_id, "Handshaking".to_string())
            .await;
    }

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
    jwt: Option<&'static str>,
    remote_host: Option<&'static str>,
    remote_port: Option<&'static str>,
    local_ip: Option<&'static str>,
    local_port: Option<&'static str>,
    opts: Option<&'static str>,
    mode: OperationMode,
}

enum OperationMode {
    Server,
    Client(u32),
}

async fn get_run_params() -> RunParams {
    let remote_host: Option<&'static str> = option_env!("SS_REMOTE_HOST");
    let remote_port: Option<&'static str> = option_env!("SS_REMOTE_PORT");
    let local_ip: Option<&'static str> = option_env!("SS_LOCAL_HOST");
    let local_port: Option<&'static str> = option_env!("SS_LOCAL_PORT");
    let opts: Option<&'static str> = option_env!("SS_PLUGIN_OPTIONS");
    let mode: OperationMode = OperationMode::Server;

    // TODO: Figure out operation mode from shadowsocks env

    let mut phone_number: Option<u64> = None;
    let mut jwt: Option<&'static str> = None;
    opts.map(|opts| {
        opts.split(';')
            .map(|opt| opt.trim())
            .filter(|opt| {
                if opt.to_lowercase().starts_with("phone_number=") {
                    phone_number = opt["phone_number=".len()..].parse().ok();
                    return false;
                } else if opt.to_lowercase().starts_with("jwt=") {
                    jwt = Some(&opt["jwt=".len()..]);
                    return false;
                }
                true
            })
            .collect::<Vec<&str>>()
            .join(";")
    });

    let running_from_shadowsocks = matches!(
        (remote_host, remote_port, local_ip, local_port),
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
        local_ip,
        local_port,
        opts,
        mode,
    }
}
