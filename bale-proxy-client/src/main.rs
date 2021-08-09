use bale::BaleClient;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() {
    let phone_number_str = get_input("Please enter mobile number (can be from receive-smss.com): ");

    let mut bale = BaleClient::new();
    let registered = bale.login(phone_number_str.parse().unwrap()).await;

    let login_code = get_input("Please enter login code sent by sms: ");
    bale.validate_code(&login_code).await;
}

fn get_input(message: &str) -> String {
    let mut s = String::new();
    print!("{}", message);
    let _ = stdout().flush();
    stdin()
        .read_line(&mut s)
        .expect("Did not enter a correct string");
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    s
}
