use rocket::response::status::Unauthorized;

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

async fn login(username: &str, password: &str, client_option: Option<reqwest::Client>) -> Result<reqwest::Client, Unauthorized<String>> {
    let client = client_option.unwrap_or(reqwest::Client::builder().cookie_store(true).build().map_err(|_| Unauthorized(Some("Unable to build the client".to_owned())))?);
    let login_url = "https://cas.sustech.edu.cn/cas/login";
    let mut execution = "".to_owned();
    // Get the execution
    {
        let cas_html = client.get(login_url)
                        .send()
                        .await.map_err(|_| Unauthorized(Some(String::from("Unable to send request to cas page"))))?
                        .text()
                        .await.map_err(|_| Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;
        let cas_fragment = scraper::Html::parse_fragment(&cas_html[..]);
        let input_selector = scraper::Selector::parse("input").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                    
        let inputs = cas_fragment.select(&input_selector);
        for input in inputs {
            if input.value().attr("name").unwrap_or_default() == "execution" {
                execution = input.value().attr("value").unwrap_or_default().to_owned();
            }
        }
    }

    {
        // let login_post_body = format!("username={}&password={}&execution={}&_eventId=submit", username, password, execution);
        let login_post_form = [
                                ("username", username), 
                                ("password", password),
                                ("execution", &execution),
                                ("_eventId", "submit"),
                                ("locale", "en")];
        let login_resp = client.post(login_url).form(&login_post_form).send()
                                .await.map_err(|_| Unauthorized(Some(String::from("Unable to send request to cas page"))))?;

        let login_html = login_resp.text()
                                    .await.map_err(|_| Unauthorized(Some("Unable to convert the response to HTML text".to_owned())))?;
        if login_html.contains("Log In Successful") {
            println!("{:?}", client);
            Ok(client)
        } else {
            Err(Unauthorized(Some("Login Failed".to_owned())))
        }
        
        
    }
}

#[rocket::get("/cas_login?<username>&<password>")]
async fn cas_login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let client = login(username, password, None).await?;
    println!("{:?}", client);
    Ok("Hello world!".to_owned())
}

#[rocket::get("/tis_login?<username>&<password>")]
async fn tis_login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let mut client = reqwest::Client::builder().cookie_store(true).gzip(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0").build().map_err(|_| Unauthorized(Some("Unable to build the client".to_owned())))?;
    let tis_cas_url = "https://cas.sustech.edu.cn/cas/login?service=https://tis.sustech.edu.cn/cas";
    client = login(username, password, Some(client)).await.map_err(|_| Unauthorized(Some("Unable to login".to_owned())))?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://tis.sustech.edu.cn/"));
    headers.insert("Accept", reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("zh-cn"));
    headers.insert("Accept-Encoding", reqwest::header::HeaderValue::from_static("gzip, deflate, br"));

    client.get(tis_cas_url).headers(headers).send()
            .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    let tis_cas_reps = client.get("https://tis.sustech.edu.cn/user/basic").send().await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
    tis_cas_reps.text().await.map_err(|_| Unauthorized(Some("Unable to parse the tis_cas_reps to text".to_owned())))
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(cas_login))
                    .mount("/", rocket::routes!(tis_login))
}
