use rocket::response::status::Unauthorized;

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

#[rocket::get("/login?<username>&<password>")]
async fn login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let client = reqwest::Client::new();
    let login_url = "https://cas.sustech.edu.cn/cas/login";
    let mut exection: Option<&str> = None;

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
                exection = input.value().attr("value").clone();
            }
        }

    

    if let Some(exection) = exection {
        
        let login_post_body = format!("username={}&password={}&execution={}&_eventId=submit", username, password, exection);
        let login_html = client.post(login_url).body(login_post_body).send()
                                .await.map_err(|_| Unauthorized(Some(String::from("Unable to send request to cas page"))))?
                                .text()
                                .await.map_err(|_| Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;

        Ok(login_html)

    } else {
        Err(Unauthorized(Some(String::from("Unable to find the execution code"))))
    }
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(login))
}
