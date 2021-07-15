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
    id: String,
    points: u32,
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

use std::{collections::HashMap};
use rocket::{State, response::status::{self, Unauthorized}};
use scraper::{Html, Selector};
use futures::lock::Mutex;

const USER_AGENT: &'static str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0";
const LOGIN_URL: &'static str = "https://cas.sustech.edu.cn/cas/login";
const TIS_CAS_URL: &'static str = "https://cas.sustech.edu.cn/cas/login?service=https://tis.sustech.edu.cn/cas";
const BASIC_INFO_URL: &'static str = "https://tis.sustech.edu.cn/UserManager/queryxsxx";
const SEMESTER_GPA_URL: &'static str = "https://tis.sustech.edu.cn/cjgl/xscjgl/xsgrcjcx/queryXnAndXqXfj";
const COURSE_GRADES_URL: &'static str = "https://tis.sustech.edu.cn/cjgl/grcjcx/grcjcx";
const COURSES_URL: &'static str = "https://course-tao.sustech.edu.cn/kcxxweb/KcxxwebChinesePC";
const SELECTED_COURSES_URL: &'static str = "https://tis.sustech.edu.cn/Xsxk/queryYxkc";
const AVAILABLE_COURSES_URL: &'static str = "https://tis.sustech.edu.cn/Xsxk/queryKxrw";
const SELECT_COURSE_URL: &'static str = "https://tis.sustech.edu.cn/Xsxk/addGouwuche"; // WTF???? 购物车？？？
const DROP_COURSE_URL: &'static str = "https://tis.sustech.edu.cn/Xsxk/tuike";
const UPDATE_POINTS_URL: &'static str = "https://tis.sustech.edu.cn/Xsxk/updXkxsByyx";

async fn use_client_login(
    client: &reqwest::Client
) -> Result<bool, status::Unauthorized<String>> {
    let post_form = [("locale", "en")];
    let resp = client.post(LOGIN_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| status::Unauthorized(Some(String::from("Receive response from CAS failed!"))))?
                                    .text()
                                    .await
                                    .map_err(|_| status::Unauthorized(Some(String::from("Parse the reponse to string failed!"))))?;
    
    return Ok(resp.contains("Log In Successful"));
}

async fn use_username_password_login(
    client: &reqwest::Client,
    username: &str,
    password: &str,
    execution: &str,
) -> Result<bool, status::Unauthorized<String>> {
    let post_form = [ 
        ("username", username), 
        ("password", password),
        ("execution", execution),
        ("_eventId", "submit"),
        ("locale", "en")];
    let login_resp_html = client.post(LOGIN_URL)
                                        .form(&post_form)
                                        .send()
                                        .await
                                        .map_err(|_| status::Unauthorized(Some(String::from("Get the login response failed!"))))?
                                        .text()
                                        .await
                                        .map_err(|_| status::Unauthorized(Some(String::from("Parse the response to text failed!"))))?;
    #[cfg(debug_assertions)] {
        println!("{}", login_resp_html);
        println!("{:?}", post_form);
    }
    
    Ok(login_resp_html.contains("Log In Successful"))
}

async fn get_execution_code(
    client: &reqwest::Client
) -> Result<String, status::Unauthorized<String>> {
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
        return Ok(input.value().attr("value").unwrap().to_owned());
    } else {
        return Err(status::Unauthorized(Some(String::from("Cannot find the input with execution code"))));
    }
}

async fn login(
    username: &str, 
    password: &str, 
    client_storage: &Mutex<HashMap<String, reqwest::Client>>
) -> Result<bool, status::Unauthorized<String>> {
    let mut new_client_flag = false;
    let mut client_storage = client_storage.lock().await;
    if !client_storage.contains_key(username) {
        client_storage.insert(
            username.to_owned(), 
            reqwest::Client::builder()
                                .user_agent(USER_AGENT)
                                .cookie_store(true)
                                .build()
                                .map_err(|_| Unauthorized(Some(String::from("Build new client for the user failed!"))))?
        );
        new_client_flag = true;
    }

    let client = client_storage.get(username).unwrap();

    if !new_client_flag && 
        use_client_login(&client).await.unwrap() 
    {
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

async fn tis_login(
    username: &str, 
    password: &str,
    client_storage: &Mutex<HashMap<String, reqwest::Client>>
) -> Result<bool, Unauthorized<String>> {
    if !login(username, password, &client_storage).await.unwrap() {
        return Ok(false);
    }
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://tis.sustech.edu.cn/"));
    headers.insert("Accept", reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("zh-cn"));
    headers.insert("Accept-Encoding", reqwest::header::HeaderValue::from_static("gzip, deflate, br"));

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();
    client.get(TIS_CAS_URL)
            .headers(headers)
            .send()
            .await
            .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    Ok(true)
}

#[rocket::get("/")]
pub async fn index() -> String {
    String::from("Hello, rustech!\n")
}

#[rocket::get("/cas_login?<username>&<password>")]
pub async fn cas_login(
    username: &str, 
    password: &str, 
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {
    if login(username, password, client_storage)
        .await?
    {
        return Ok(String::from("Login Successfully!"));
    }
    return Err(Unauthorized(Some(String::from("Login Failed!"))));
}


#[rocket::get("/basic_info?<username>&<password>")]
pub async fn basic_info(
    username: &str, 
    password: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();
    
    let v = client.post(BASIC_INFO_URL)
                        .send()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
    let basic_info = BasicInfo {
        id: v["ID"].as_str().unwrap_or_default().to_owned(),
        sid: v["XH"].as_str().unwrap_or_default().to_owned(),
        name: v["XM"].as_str().unwrap_or_default().to_owned(),
        email: v["DZYX"].as_str().unwrap_or_default().to_owned(),
        year: v["NJMC"].as_str().unwrap_or_default().to_owned(),
        department: v["YXMC"].as_str().unwrap_or_default().to_owned(),
        major: v["ZYMC"].as_str().unwrap_or_default().to_owned()
    };

    Ok(serde_json::to_string_pretty(&basic_info).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/semester_gpa?<username>&<password>")]
pub async fn semester_gpa(
    username: &str, 
    password: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();
    
    let v = client.post(SEMESTER_GPA_URL)
                        .send()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
    let gpa_value_array = v["xnanxqxfj"].as_array().unwrap();
    let mut gpa_vec = Vec::<SemesterGPA>::new();
    for gpa in gpa_value_array {
        let semester_gpa = SemesterGPA {
            semester_full_name: gpa["XNXQ"].as_str().unwrap_or_default().to_owned(),
            semester_year: gpa["XN"].as_str().unwrap_or_default().to_owned(),
            semester_number: gpa["XQ"].as_str().unwrap_or_default().to_owned(),
            gpa: gpa["XQXFJ"].as_f64()
        };
        gpa_vec.push(semester_gpa);
    }

    let student_gpa = StudentGPA {
        all_gpa: gpa_vec,
        average_gpa: v["xfjandpm"]["PJXFJ"].as_f64().unwrap_or_default(),
        rank: v["xfjandpm"]["PM"].as_str().unwrap_or_default().to_owned()
    };
    
    Ok(serde_json::to_string_pretty(&student_gpa).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/courses_grades?<username>&<password>")]
pub async fn courses_grades(
    username: &str, 
    password: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
    let body = r#"{"xn":null,"xq":null,"kcmc":null,"cxbj":"-1","pylx":"1","current":1,"pageSize":100}"#;
    let v: serde_json::Value = client.post(COURSE_GRADES_URL)
                                    .headers(headers)
                                    .body(body)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
    let course_grades_value_vec = v["content"]["list"].as_array().unwrap();
    let mut course_grades_vec = Vec::<CourseGrade>::new();
    for course_grade_value in course_grades_value_vec {
        let course_grade = CourseGrade {
            code: course_grade_value["kcdm"].as_str().unwrap_or_default().to_owned(),
            name: course_grade_value["kcmc"].as_str().unwrap_or_default().to_owned(),
            class_hour: course_grade_value["xs"].as_str().unwrap_or_default().to_owned(),
            credit: course_grade_value["xf"].as_u64().unwrap_or_default().to_owned(),
            semester: course_grade_value["xnxqmc"].as_str().unwrap_or_default().to_owned(),
            final_grade: course_grade_value["zzcj"].as_str().unwrap_or_default().to_owned(),
            final_level: course_grade_value["xscj"].as_str().unwrap_or_default().to_owned(),
            department: course_grade_value["yxmc"].as_str().unwrap_or_default().to_owned(),
        };
        course_grades_vec.push(course_grade);
    }

    #[cfg(debug_assertions)]
    println!("Total {} course grades item", course_grades_vec.len());

    Ok(serde_json::to_string_pretty(&course_grades_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/courses")]
pub async fn get_courses() -> Result<String, Unauthorized<String>> {
    
    let courses_html = reqwest::get(COURSES_URL)
                                        .await
                                        .map_err(|_| Unauthorized(Some("Unable to get courses from the web".to_owned())))?
                                        .text()
                                        .await
                                        .map_err(|_| Unauthorized(Some("Unable to get courses from the web".to_owned())))?;

    let cas_fragment = scraper::Html::parse_fragment(&courses_html[..]);
    let table_selector = scraper::Selector::parse("table")
                                                    .map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                    
    let mut table_iter = cas_fragment.select(&table_selector);
    let _head_table = table_iter.next().unwrap();

    let tr_selector = scraper::Selector::parse("tr")
                                                .map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let td_selector = scraper::Selector::parse("td")
                                                .map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let a_selector = scraper::Selector::parse("a")
                                                .map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let mut courses_vec = Vec::<Course>::new();
    for table in table_iter {
        let mut tr_iter = table.select(&tr_selector);
        tr_iter.next();
        for tr in tr_iter {
            let mut td_iter = tr.select(&td_selector);
            let course = Course {
                course_id: td_iter.next().unwrap().select(&a_selector).next().unwrap().inner_html(),
                course_name: td_iter.next().unwrap().select(&a_selector).next().unwrap().inner_html(),
                credits: td_iter.next().unwrap().inner_html().parse::<f32>().unwrap(),
                department: td_iter.last().unwrap().inner_html()
            };
            courses_vec.push(course);
        }
    }
    Ok(serde_json::to_string_pretty(&courses_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/selected_courses?<username>&<password>&<semester_year>&<semester_no>")]
pub async fn selected_courses(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("p_xkfsdm", "yixuan");
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    let v = client.post(SELECTED_COURSES_URL)
                        .form(&post_form)
                        .send()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                        .json::<serde_json::Value>()
                        .await
                        .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    let selected_courses_value = v["yxkcList"].as_array().unwrap();
    let mut selected_courses_vec = Vec::<SelectedCourse>::new();

    for value in selected_courses_value {
        let course = SelectedCourse {
            basic_course: Course {
                course_id: value["kcdm"].as_str().unwrap().to_owned(),
                course_name: value["kcmc"].as_str().unwrap().to_owned(),
                credits: value["xf"].as_str().unwrap().parse::<f32>().map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?,
                department: value["kkyxmc"].as_str().unwrap().to_owned()
            },
            course_class: value["rwmc"].as_str().unwrap().to_owned(),
            course_type: value["kclbmc"].as_str().unwrap().to_owned(),
            id: value["id"].as_str().unwrap().to_owned(),
            available: match value["sxbj"].as_str().unwrap() {
                "0" => { false },
                "1" => { true },
                _ => {false}
            },
            teacher: value["dgjsmc"].as_str().unwrap().to_owned(),
            time_and_place: {
                let course_info_html = value["kcxx"].as_str().unwrap();
                let course_info_fragment = scraper::Html::parse_fragment(&course_info_html);
                let div_selector = scraper::Selector::parse("div").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                let mut div_iter = course_info_fragment.select(&div_selector);
                let p_selector = scraper::Selector::parse("p").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                let p = div_iter.next().unwrap().select(&p_selector).next().unwrap().inner_html();
                p
            },
            points: value["xkxs"].as_str().unwrap().parse::<u32>().unwrap(),
        };
        selected_courses_vec.push(course);
    }
    Ok(serde_json::to_string_pretty(&selected_courses_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/available_courses?<username>&<password>&<semester_year>&<semester_no>&<courses_type>")]
pub async fn available_courses(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    courses_type: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    let code_p_xkfsdm = match courses_type {
        "GR" => "bxxk", //  General Required
        "GE" => "xxxk", //  General Elective
        "TP" => "kzyxk", //  Courses within the training program
        "NTP" => "zynknjxk", //  Courses without the training program
        _ => ""
    };

    post_form.insert("p_xkfsdm", code_p_xkfsdm);
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    let v: serde_json::Value = client.post(AVAILABLE_COURSES_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    let available_courses_value = v["kxrwList"]["list"].as_array().unwrap();
    
    let mut available_courses_vec = Vec::<AvailableCourse>::new();

    for value in available_courses_value {
        let course = AvailableCourse {
            basic_course: Course {
                course_id: value["kcdm"].as_str().unwrap().to_owned(),
                course_name: value["kcmc"].as_str().unwrap().to_owned(),
                credits: value["xf"].as_str().unwrap().parse::<f32>().map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?,
                department: value["kkyxmc"].as_str().unwrap().to_owned(),
            },
            course_class: value["rwmc"].as_str().unwrap().to_owned(),
            course_type: value["kclbmc"].as_str().unwrap().to_owned(),
            teacher: value["dgjsmc"].as_str().unwrap().to_owned(),
            id: value["id"].as_str().unwrap().to_owned(),
            time_and_place: {
                let course_info_html = value["kcxx"].as_str().unwrap();
                let course_info_fragment = scraper::Html::parse_fragment(&course_info_html);
                let div_selector = scraper::Selector::parse("div").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                let mut div_iter = course_info_fragment.select(&div_selector);
                let p_selector = scraper::Selector::parse("p").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                let p = div_iter.next().unwrap().select(&p_selector).next().unwrap().inner_html();
                p
            }
        };
        available_courses_vec.push(course);
    }
    Ok(serde_json::to_string_pretty(&available_courses_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/select_course?<username>&<password>&<semester_year>&<semester_no>&<course_id>&<course_type>&<points>")]
pub async fn select_course(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    course_id: &str, 
    course_type: &str, 
    points: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let code_p_xkfsdm = match course_type {
        "GR" => "bxxk", //  General Required
        "GE" => "xxxk", //  General Elective
        "TP" => "kzyxk", //  Courses within the training program
        "NTP" => "zynknjxk", //  Courses without the training program
        _ => ""
    };
    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    post_form.insert("p_id", course_id);
    post_form.insert("p_xkxs", points);
    post_form.insert("p_pylx", "1");
    post_form.insert("p_xkfsdm", code_p_xkfsdm);
    post_form.insert("p_xktjz", "rwtjzyx");

    let v: serde_json::Value = client.post(SELECT_COURSE_URL)
                                    .form(&post_form)
                                    .send()
                                    .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    #[cfg(debug_assertions)]
    println!("{}", serde_json::to_string_pretty(&v).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?);
    if v["gjhczztm"].as_str().unwrap() == "OPERATE.RESULT_SUCCESS" {
        return Ok("SUCCESS".to_owned());
    } else {
        return Err(Unauthorized(Some(v["message"].as_str().unwrap().to_owned())));
    }
}

#[rocket::get("/drop_course?<username>&<password>&<semester_year>&<semester_no>&<course_id>")]
pub async fn drop_course(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    course_id: &str, 
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    post_form.insert("p_id", course_id);
    post_form.insert("p_pylx", "1");
    post_form.insert("p_xkfsdm", "yixuan");

    let v: serde_json::Value = client.post(DROP_COURSE_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    #[cfg(debug_assertions)]
    println!("{}", serde_json::to_string_pretty(&v).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?);
    if v["gjhczztm"].as_str().unwrap() == "OPERATE.RESULT_SUCCESS" {
        return Ok("SUCCESS".to_owned());
    } else {
        return Err(Unauthorized(Some(v["message"].as_str().unwrap().to_owned())));
    }
}

#[rocket::get("/update_points?<username>&<password>&<semester_year>&<semester_no>&<course_id>&<points>")]
pub async fn update_points(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    course_id: &str, 
    points: &str,
    client_storage: &State<Mutex<HashMap<String, reqwest::Client>>>
) -> Result<String, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Ok(String::from("Login to the tis system failed!")); }

    let client_storage = client_storage.lock().await;
    let client = client_storage.get(username).unwrap();

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    post_form.insert("p_id", course_id);
    post_form.insert("p_pylx", "1");
    post_form.insert("p_xkfsdm", "yixuan");
    post_form.insert("p_xkxs", points);

    let v: serde_json::Value = client.post(UPDATE_POINTS_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    #[cfg(debug_assertions)]
    println!("{}", serde_json::to_string_pretty(&v).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?);
    if v["jg"].as_str().unwrap() == "1" {
        return Ok("SUCCESS".to_owned());
    } else {
        return Err(Unauthorized(Some(v["message"].as_str().unwrap().to_owned())));
    }
}

#[cfg(test)]
mod tests {
    use futures::lock::Mutex;

    use rocket::tokio;

    #[tokio::test]
    async fn test_use_client_login() {
        let client = reqwest::Client::builder()
                                                .cookie_store(true)
                                                .user_agent(super::USER_AGENT)
                                                .build()
                                                .unwrap();
        let mut username = String::new();
        let mut password = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        std::io::stdin().read_line(&mut password).unwrap();
        username = username.strip_suffix("\n").unwrap().to_owned();
        password = password.strip_suffix("\n").unwrap().to_owned();
        let execution = super::get_execution_code(&client).await.unwrap();

        println!("{:?}", super::use_username_password_login(&client, &username, &password, &execution).await.unwrap());
        println!("{:?}", super::use_client_login(&client).await.unwrap());                                            
    }

    #[tokio::test]
    async fn test_username_password_login() {
        let client = reqwest::Client::builder()
                                            .user_agent(super::USER_AGENT)
                                            .build()
                                            .unwrap();
        let mut username = String::new();
        let mut password = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        std::io::stdin().read_line(&mut password).unwrap();
        username = username.strip_suffix("\n").unwrap().to_owned();
        password = password.strip_suffix("\n").unwrap().to_owned();
        let execution = super::get_execution_code(&client).await.unwrap();

        println!("{:?}", super::use_username_password_login(&client, &username, &password, &execution).await.unwrap())
    }

    #[tokio::test]
    async fn test_get_execution_code() {
        let client = reqwest::Client::builder()
                                            .user_agent(super::USER_AGENT)
                                            .build()
                                            .unwrap();
        let future = super::get_execution_code(&client);
        let result = future.await;
        println!("{:?}", result.unwrap());
    }

    #[tokio::test]
    async fn test_login() {
        let mut username = String::new();
        let mut password = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        std::io::stdin().read_line(&mut password).unwrap();
        username = username.strip_suffix("\n").unwrap().to_owned();
        password = password.strip_suffix("\n").unwrap().to_owned();
        let client_storage = Mutex::new(std::collections::HashMap::new());
        println!("The first login result: {:?}", super::login(&username, &password, &client_storage).await.unwrap());
        println!("The second login result: {:?}", super::login(&username, &password, &client_storage).await.unwrap());
    }

    #[tokio::test]
    async fn test_tis_login() {
        let mut username = String::new();
        let mut password = String::new();
        std::io::stdin().read_line(&mut username).unwrap();
        std::io::stdin().read_line(&mut password).unwrap();
        username = username.strip_suffix("\n").unwrap().to_owned();
        password = password.strip_suffix("\n").unwrap().to_owned();
        let client_storage = Mutex::new(std::collections::HashMap::new());
        println!("The first login result: {:?}", super::tis_login(&username, &password, &client_storage).await.unwrap());
        println!("The second login result: {:?}", super::tis_login(&username, &password, &client_storage).await.unwrap());
    }
}