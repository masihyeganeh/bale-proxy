use bale::BaleClient;
use std::io::{stdin, stdout, Write};

mod error;
mod simple_server;
use simple_server::get_from_web;

#[tokio::main]
async fn main() {
    let run_params = get_run_params().await;

    eprintln!("Run params : {:#?}", run_params);

    let phone_number = run_params.phone_number.expect("bad phone number");

    let mut bale = BaleClient::new();
    let registered = bale.login(phone_number).await;

    if !registered {
        panic!("phone number is not registered")
    }

    let login_code = get_input(
        format!("Please enter login code sent by sms to {}:", phone_number),
        !run_params.running_from_shadowsocks,
    )
    .await;
    bale.validate_code(&login_code).await;

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
    remote_host: Option<&'static str>,
    remote_port: Option<&'static str>,
    local_ip: Option<&'static str>,
    local_port: Option<&'static str>,
    opts: Option<&'static str>,
}

async fn get_run_params() -> RunParams {
    let remote_host: Option<&'static str> = option_env!("SS_REMOTE_HOST");
    let remote_port: Option<&'static str> = option_env!("SS_REMOTE_PORT");
    let local_ip: Option<&'static str> = option_env!("SS_LOCAL_HOST");
    let local_port: Option<&'static str> = option_env!("SS_LOCAL_PORT");
    let opts: Option<&'static str> = option_env!("SS_PLUGIN_OPTIONS");

    let mut phone_number: Option<u64> = None;
    opts.map(|opts| {
        opts.split(';')
            .map(|opt| opt.trim())
            .filter(|opt| {
                if opt.to_lowercase().starts_with("phone_number=") {
                    phone_number = opt["phone_number=".len()..].parse().ok();
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
        remote_host,
        remote_port,
        local_ip,
        local_port,
        opts,
    }
}
