use std::collections::HashMap;

use futures::lock::Mutex;
use rocket::response::status::Unauthorized;
use scraper::{Html, Selector};
use super::urls::*;

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
    client_storage: &Mutex<HashMap<String, reqwest::Client>>,
) -> Result<bool, Unauthorized<String>> {
    let mut new_client_flag = false;
    let mut client_storage = client_storage.lock().await;
    if !client_storage.contains_key(username) {
        client_storage.insert(
            username.to_owned(),
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .cookie_store(true)
                .build()
                .map_err(|_| {
                    Unauthorized(Some(String::from("Build new client for the user failed!")))
                })?,
        );
        new_client_flag = true;
    }
    let client = client_storage.get(username).unwrap();

    if !new_client_flag && use_client_login(&client).await.unwrap() {
        #[cfg(debug_assertions)]
        println!("The login reuse the old client");
        return Ok(true);
    } else {
        #[cfg(debug_assertions)]
        println!("The login new the new client");
        let execution = get_execution_code(&client).await.unwrap();
        return use_username_password_login(&client, &username, &password, &execution).await;
    }
}

pub async fn tis_login(
    username: &str,
    password: &str,
    client_storage: &Mutex<HashMap<String, reqwest::Client>>,
) -> Result<bool, Unauthorized<String>> {
    if !login(username, password, &client_storage).await.unwrap() {
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

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();
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
