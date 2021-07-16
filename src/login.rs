use std::collections::HashMap;

use futures::lock::Mutex;
use rocket::response::status::Unauthorized;
use scraper::{Html, Selector};
use crate::encrypt::{encrypt, generate_salt, verify};

use super::urls::*;
use super::structures::Account;

pub async fn use_client_login(client: &reqwest::Client) -> Result<bool, Unauthorized<String>> {
    let post_form = [("locale", "en")];
    let resp = client
        .post(LOGIN_URL)
        .form(&post_form)
        .send()
        .await
        .map_err(|_| Unauthorized(Some(String::from("Receive response from CAS failed!"))))?
        .text()
        .await
        .map_err(|_| {
            Unauthorized(Some(String::from("Parse the reponse to string failed!")))
        })?;

    return Ok(resp.contains("Log In Successful"));
}

pub async fn use_username_password_login(
    client: &reqwest::Client,
    username: &str,
    password: &str,
    execution: &str,
) -> Result<bool, Unauthorized<String>> {
    let post_form = [
        ("username", username),
        ("password", password),
        ("execution", execution),
        ("_eventId", "submit"),
        ("locale", "en"),
    ];
    let login_resp_html = client
        .post(LOGIN_URL)
        .form(&post_form)
        .send()
        .await
        .map_err(|_| Unauthorized(Some(String::from("Get the login response failed!"))))?
        .text()
        .await
        .map_err(|_| {
            Unauthorized(Some(String::from("Parse the response to text failed!")))
        })?;
    #[cfg(debug_assertions)]
    {
        println!("{}", login_resp_html);
        println!("{:?}", post_form);
    }

    Ok(login_resp_html.contains("Log In Successful"))
}

pub async fn get_execution_code(
    client: &reqwest::Client,
) -> Result<String, Unauthorized<String>> {
    let cas_html = client
        .get(LOGIN_URL)
        .send()
        .await
        .map_err(|_| {
            Unauthorized(Some(String::from("Unable to send request to cas page")))
        })?
        .text()
        .await
        .map_err(|_| {
            Unauthorized(Some(String::from("Unable to the response to HTML text")))
        })?;
    let cas_fragment = Html::parse_fragment(&cas_html);
    let input_selector = Selector::parse("input").map_err(|_| {
        Unauthorized(Some(String::from("Unable to parse HTML to fragment")))
    })?;
    let inputs = cas_fragment.select(&input_selector);
    let execution_code_input = inputs
        .filter(|e| e.value().attr("name").unwrap_or_default() == "execution")
        .next();
    if let Some(input) = execution_code_input {
        return Ok(input.value().attr("value").unwrap().to_owned());
    } else {
        return Err(Unauthorized(Some(String::from(
            "Cannot find the input with execution code",
        ))));
    }
}

pub async fn login(
    username: &str,
    password: &str,
    client_storage: &Mutex<HashMap<String, Account>>,
) -> Result<bool, Unauthorized<String>> {
    let mut account_storage = client_storage.lock().await;
    if !account_storage.contains_key(username) {
        account_storage.insert(
            username.to_owned(),
            Account {
                hash_salt: None,
                client: reqwest::Client::builder()
                                        .user_agent(USER_AGENT)
                                        .cookie_store(true)
                                        .build()
                                        .map_err(|_| Unauthorized(Some(String::from("Build new client for the user failed!"))))?,
            }
        );
    }
    let mut account = account_storage.get_mut(username).unwrap();
    let client = &account.client;

    if let Some(hash_salt) = &mut account.hash_salt {
        if verify(password, &hash_salt.0, &hash_salt.1) {
            if use_client_login(client).await.unwrap() {
                #[cfg(debug_assertions)]
                println!("Password correct, and use the old client login successfully!");
                return Ok(true);
            } else {
                #[cfg(debug_assertions)]
                println!("Password correct, and use the old client login failed, try using password to login.");
                let execution = get_execution_code(client).await.unwrap();
                if !use_username_password_login(client, &username, &password, &execution).await? {
                    #[cfg(debug_assertions)]
                    println!("\tUse password to login failed! (PASSWORD CHANGED, USE OLD PASSWORD)");
                    return Err(Unauthorized(Some(String::from("Login failed! Have you changed the password?"))));
                } else {
                    #[cfg(debug_assertions)]
                    println!("\tUse password to login successfully!");
                    return Ok(true);
                }
            }
        } else {
            #[cfg(debug_assertions)]
            println!("Password check failed (PASSWORD MAY CHANGED, USE NEW PASSWORD)");
            let old_client = account.client.clone();
            account.client = reqwest::Client::builder()
                            .user_agent(USER_AGENT)
                            .cookie_store(true)
                            .build()
                            .map_err(|_| Unauthorized(Some(String::from("Build new client for the user failed!"))))?;
            let client = &account.client;
            let execution = get_execution_code(client).await.unwrap();
            if use_username_password_login(client, &username, &password, &execution).await? {
                #[cfg(debug_assertions)]
                println!("\tUse password to login successfully, UPDATE HASH_SALT");
                hash_salt.1 = generate_salt().unwrap();
                hash_salt.0 = encrypt(password, &hash_salt.1);
                return Ok(true);
            } else {
                #[cfg(debug_assertions)]
                println!("\tUse password to login failed, NOT UPDATE HASH_SALT");
                account.client = old_client;
                return Err(Unauthorized(Some(String::from("Login failed!"))));
            }
        }
    } else {
        #[cfg(debug_assertions)]
        println!("New client login!");
        let execution = get_execution_code(client).await.unwrap();
        if use_username_password_login(client, &username, &password, &execution).await? {
            #[cfg(debug_assertions)]
            println!("\tUse password to login successfully, UPDATE HASH_SALT");               
            account.hash_salt = {
                let salt = generate_salt().unwrap();
                let hash = encrypt(password, &salt);
                Some((hash, salt))
            };
            return Ok(true);
        } else {
            #[cfg(debug_assertions)]
            println!("\tUse password to login failed, NOT UPDATE HASH_SALT");
            return Err(Unauthorized(Some(String::from("Login failed!"))));
        }
    }
}

pub async fn tis_login(
    username: &str,
    password: &str,
    client_storage: &Mutex<HashMap<String, Account>>,
) -> Result<bool, Unauthorized<String>> {
    if !login(username, password, &client_storage).await? {
        return Ok(false);
    }
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Referer",
        reqwest::header::HeaderValue::from_static("https://tis.sustech.edu.cn/"),
    );
    headers.insert(
        "Accept",
        reqwest::header::HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        ),
    );
    headers.insert(
        "Accept-Language",
        reqwest::header::HeaderValue::from_static("zh-cn"),
    );
    headers.insert(
        "Accept-Encoding",
        reqwest::header::HeaderValue::from_static("gzip, deflate, br"),
    );

    let account_storage = client_storage.lock().await;
    let account = account_storage.get(username).unwrap();
    let client = &account.client;
    client
        .get(TIS_CAS_URL)
        .headers(headers)
        .send()
        .await
        .map_err(|_| {
            Unauthorized(Some(
                "Unable to send the login redirect request to CAS".to_owned(),
            ))
        })?;

    Ok(true)
}
