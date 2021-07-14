#[derive(serde::Serialize, serde::Deserialize)]
struct BasicInfo {
    id: String,
    sid: String,
    name: String,
    email: String,
    year: String,
    department: String,
    major: String
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SemesterGPA {
    semester_full_name: String,
    semester_year: String,
    semester_number: String,
    gpa: Option<f64>
}

#[derive(serde::Serialize, serde::Deserialize)]
struct StudentGPA {
    all_gpa: std::vec::Vec<SemesterGPA>,
    average_gpa: f64,
    rank: String
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CourseGrade {
    code: String,
    name: String,
    class_hour: String,
    credit: u64,
    semester: String,
    final_grade: String,
    final_level: String,
    department: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Course {
    course_id: String,
    course_name: String,
    credits: f32,
    department: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SelectedCourse {
    basic_course: Course,
    course_type: String,
    course_class: String,
    teacher: String,
    time_and_place: String,
    available: bool,
    id: String
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AvailableCourse {
    basic_course: Course,
    course_type: String,
    course_class: String,
    teacher: String,
    time_and_place: String,
    id: String
}

use std::collections::HashMap;
use rocket::response::status;
use scraper::{Html, Selector};

const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0";
const LOGIN_URL: &'static str = "https://cas.sustech.edu.cn/cas/login";

#[rocket::get("/")]
async fn index() -> String {
    String::from("Hello, rustech!\n")
}

async fn use_client_login(
    client: reqwest::Client
) -> Result<bool, status::Unauthorized<String>> 
{
    let post_form = [("locale", "en")];
    let resp = client.post(LOGIN_URL)
                                    .send()
                                    .await
                                    .map_err(|_| status::Unauthorized(Some(String::from("Receive response from CAS failed!"))))?
                                    .text()
                                    .await
                                    .map_err(|_| status::Unauthorized(Some(String::from("Parse the reponse to string failed!"))))?;
    
    return Ok(resp.contains("Log In Successful"));
}

async fn get_execution_code(
    client: reqwest::Client
) -> Result<String, status::Unauthorized<String>> 
{
    let cas_html = client.get(LOGIN_URL)
                                .send()
                                .await
                                .map_err(|_| status::Unauthorized(Some(String::from("Unable to send request to cas page"))))?
                                .text()
                                .await
                                .map_err(|_| status::Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;
    let cas_fragment = Html::parse_fragment(&cas_html);
    let input_selector = Selector::parse("input")
                                            .map_err(|_| status::Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;                
    let inputs = cas_fragment.select(&input_selector);
    let execution_code_input = inputs.filter(|e| e.value().attr("name").unwrap_or_default() == "execution").next();
    if let Some(input) = execution_code_input {
        return Ok(input);
    } else {
        return Err(status::Unauthorized(Some(String::from("Cannot find the input with execution code"))));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_use_client_login() {
        let client = reqwest::Client::builder()
                                                .user_agent(super::USER_AGENT)
                                                .build()
                                                .unwrap();
        
                                            
    }
}

// async fn login(
//     username: &str, 
//     password: &str, 
//     client_storage: HashMap<String, reqwest::Client>
// ) -> Result<reqwest::Client, status::Unauthorized<String>> 
// {


//     let new_client = false;
//     if !client_storage.contains_key(username) {
//         let client = reqwest::Client::builder()
//                                             .cookie_store(true)
//                                             .user_agent(USER_AGENT)
//                                             .build()
//                                             .map_err(|_| status::Unauthorized(Some(String::from("Unable to build the client"))))?;
                        
//         client_storage.insert(username.to_owned(), client);
//         new_client = true;
//     }
//     let client = client_storage.get_mut(username).unwrap();

//     if !new_client {

//     }
//     let mut execution = String::new();

//     // Get the execution
//     {
//         let cas_html = client.get(LOGIN_URL)
//                                     .send()
//                                     .await
//                                     .map_err(|_| status::Unauthorized(Some(String::from("Unable to send request to cas page"))))?
//                                     .text()
//                                     .await
//                                     .map_err(|_| status::Unauthorized(Some(String::from("Unable to the response to HTML text"))))?;

//         let cas_fragment = scraper::Html::parse_fragment(&cas_html);
//         let input_selector = scraper::Selector::parse("input")
//                                     .map_err(|_| status::Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                    
//         let inputs = cas_fragment.select(&input_selector);
//         for input in inputs {
//             if input.value().attr("name").unwrap_or_default() == "execution" {
//                 execution = input.value().attr("value").unwrap_or_default().to_owned();
//                 break;
//             }
//         }
//     }

//     {
//         // let login_post_body = format!("username={}&password={}&execution={}&_eventId=submit", username, password, execution);
//         let login_post_form = [
//                                 ("username", username), 
//                                 ("password", password),
//                                 ("execution", &execution),
//                                 ("_eventId", "submit"),
//                                 ("locale", "en")];

//         let login_resp = client.post(LOGIN_URL)
//                                         .form(&login_post_form)
//                                         .send()
//                                         .await
//                                         .map_err(|_| status::Unauthorized(Some(String::from("Unable to send request to cas page"))))?;

//         let login_html = login_resp.text()
//                                         .await
//                                         .map_err(|_| status::Unauthorized(Some(String::from("Unable to convert the response to HTML text"))))?;

//         if login_html.contains("Log In Successful") {
//             Ok(client)
//         } else {
//             Err(status::Unauthorized(Some("Login Failed".to_owned())))
//         }
//     }
// }