use rocket::response::status::Unauthorized;

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

#[rocket::get("/login")]
async fn login() -> Result<String, Unauthorized<String>> {
    let client = reqwest::Client::new();
    let login_url = "https://cas.sustech.edu.cn/cas/login";

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15".parse().unwrap());

    let cas_html = client.get(login_url).headers(headers)
                        .send()
                        .await.map_err(|e| Unauthorized(Some(e.to_string())))?
                        .text()
                        .await.map_err(|e| Unauthorized(Some(e.to_string())))?;

    

    Ok(cas_html)
}


#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(login))
}
