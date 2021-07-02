use rocket::response::status::Unauthorized;

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

#[rocket::get("/login?<username>&<password>")]
async fn login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let client = reqwest::Client::new();
    let login_url = "https://cas.sustech.edu.cn/cas/login";
    let mut execution = String::from("");

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
        
        let mut tgc: String = "".to_owned();
        {
            let login_received_cookies_borrowed = login_resp.cookies();
            for cookie in login_received_cookies_borrowed {
                if cookie.name() == "TGC" {
                    tgc = cookie.value().to_owned();
                }
            }
        }
        
        let login_html = login_resp.text()
                                    .await.map_err(|_| Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;

        if login_html.contains("Log In Successful") {
            Ok(tgc)
        } else {
            Err(Unauthorized(Some(String::from("Login Failed!"))))
        }
    }
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(login))
}
