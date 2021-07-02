use rocket::response::status::Unauthorized;

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

async fn login(username: &str, password: &str) -> Result<reqwest::cookie::Cookie, Unauthorized<String>> {
    let client = reqwest::Client::new();
    let login_url = "https://cas.sustech.edu.cn/cas/login";
    let mut execution = "".to_owned();
    // Get the execution
    {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15".parse().unwrap());
        let cas_html = client.get(login_url).headers(headers)
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
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15".parse().unwrap());
        // let login_post_body = format!("username={}&password={}&execution={}&_eventId=submit", username, password, execution);
        let login_post_form = [
                                ("username", username), 
                                ("password", password),
                                ("execution", &execution),
                                ("_eventId", "submit"),
                                ("locale", "en")];
        let login_resp = client.post(login_url).headers(headers).form(&login_post_form).send()
                                .await.map_err(|_| Unauthorized(Some(String::from("Unable to send request to cas page"))))?;
        
        let mut tgc_cookie: Option<reqwest::cookie::Cookie> = None;
        {
            let login_received_cookies_borrowed = login_resp.cookies();
            for cookie in login_received_cookies_borrowed {
                if cookie.name() == "TGC" {
                    tgc_cookie = Some(cookie);
                }
            }
        }

        if let None = tgc_cookie {
            Err(Unauthorized(Some("Unable to find tgc in cookies".to_owned())))
        } else {
            let login_html = login_resp.text()
                                    .await.map_err(|_| Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;

            if login_html.contains("Log In Successful") {
                Ok(tgc_cookie)   
            } else {
                Err(Unauthorized(Some("Login failed!".to_owned())))
            }
        }
        
        
    }
}

#[rocket::get("/cas_login?<username>&<password>")]
async fn cas_login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    login(username, password).await
}

#[rocket::get("/tis_login?<username>&<password>")]
async fn tis_login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let client = reqwest::Client::new();
    let tis_cas_url = "https://cas.sustech.edu.cn/cas/login?service=https://tis.sustech.edu.cn/cas";
    let tgc = login(username, password).await.map_err(|_| Unauthorized(Some("Unable to login".to_owned())))?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15".parse().unwrap());

    let tis_cas_reps = client.get(tis_cas_url).headers(headers).

    

    Err(Unauthorized(Some("hello".to_owned())))

}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(cas_login))
}
