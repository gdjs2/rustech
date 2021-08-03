use std::collections::HashMap;

use futures::lock::Mutex;
use rocket::{State, response::status::Unauthorized, serde::json};
use super::structures::*;
use super::urls::*;
use super::login::*;

#[rocket::get("/")]
pub async fn index() -> String {
    String::from("Hello, rustech!\n")
}

#[rocket::get("/cas_login?<username>&<password>")]
pub async fn cas_login(
    username: &str, 
    password: &str, 
    client_storage: &State<Mutex<HashMap<String, Account>>>
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
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<BasicInfo>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, &client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;
    
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

    Ok(json::Json(basic_info))
}

#[rocket::get("/semester_gpa?<username>&<password>")]
pub async fn semester_gpa(
    username: &str, 
    password: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<StudentGPA>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;
    
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
    Ok(json::Json(student_gpa))
}

#[rocket::get("/courses_grades?<username>&<password>")]
pub async fn courses_grades(
    username: &str, 
    password: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<Vec<CourseGrade>>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
            course_type: course_grade_value["kclb"].as_str().unwrap_or_default().to_owned(),
        };
        course_grades_vec.push(course_grade);
    }

    #[cfg(debug_assertions)]
    println!("Total {} course grades item", course_grades_vec.len());
    Ok(json::Json(course_grades_vec))
}

#[rocket::get("/courses")]
pub async fn get_courses(

) -> Result<json::Json<Vec<Course>>, Unauthorized<String>> {    
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
    Ok(json::Json(courses_vec))
}

async fn parse_course_info(
    course_info_html: &str
) -> (Vec<String>, Vec<String>, Option<Vec<String>>, Option<Vec<String>>) {
    let mut major_teacher = Vec::<String>::new();
    let mut major_time_and_place = Vec::<String>::new();
    let minor_teacher: Option<Vec<String>>;
    let minor_time_and_place: Option<Vec<String>>;
    
    let course_info_fragment = scraper::Html::parse_fragment(course_info_html);
    let div_selector = scraper::Selector::parse("div").unwrap();
    let p_selector = scraper::Selector::parse("p").unwrap();
    let a_selector = scraper::Selector::parse("a").unwrap();
    let mut p_iter = course_info_fragment.select(&p_selector);
    let mut div_iter = course_info_fragment.select(&div_selector);

    if course_info_fragment.select(&div_selector).count() == 2 {   
        let a_iter = p_iter.next()
                                .unwrap()
                                .select(&a_selector);
        for a in a_iter {
            major_teacher.push(a.inner_html());
        }
        let div_p_iter = div_iter.next()
                                .unwrap()
                                .select(&p_selector);
        for p in div_p_iter {
            major_time_and_place.push(p.inner_html());
        }
        minor_teacher = None;
        minor_time_and_place = None;
    } else {
        #[cfg(debug_assertions)] {
            let mut x = 0;
            for p in p_iter {
                x += 1;
                println!("{}: {}", x, p.inner_html());
            }
        }
        let mut p_iter = course_info_fragment.select(&p_selector);
        let a_iter = p_iter.nth(1)
                                .unwrap()
                                .select(&a_selector);
        for a in a_iter {
            major_teacher.push(a.inner_html());
        }
        let div_p_iter = div_iter.next()
                                .unwrap()
                                .select(&p_selector);
        for p in div_p_iter {
            major_time_and_place.push(p.inner_html());
        }
        // println!("{:?}", p_iter.nth(8).unwrap().inner_html());
        let mut minor_teacher_vec = Vec::<String>::new();
        let a_iter = p_iter.nth(7)
                                .unwrap()
                                .select(&a_selector);
        for a in a_iter {
            minor_teacher_vec.push(a.inner_html());
        }
        minor_teacher = Some(minor_teacher_vec);

        let div_p_iter = div_iter.nth(1)
                            .unwrap()
                            .select(&p_selector);
        let mut minor_time_and_place_vec = Vec::<String>::new();
        for p in div_p_iter {
            minor_time_and_place_vec.push(p.inner_html());
        }
        minor_time_and_place = Some(minor_time_and_place_vec);
    }

    return (major_teacher, major_time_and_place, minor_teacher, minor_time_and_place);
}

#[rocket::get("/selected_courses?<username>&<password>&<semester_year>&<semester_no>")]
pub async fn selected_courses(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<Vec<SelectedCourse>>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
        let (major_teacher, 
            major_time_and_place, 
            minor_teacher, 
            minor_time_and_place) = parse_course_info(value["kcxx"].as_str().unwrap()).await;
        let course = SelectedCourse {
            advanced_course: AdvancedCourse {
                basic_course: Course {
                    course_id: value["kcdm"].as_str().unwrap().to_owned(),
                    course_name: value["kcmc"].as_str().unwrap().to_owned(),
                    credits: value["xf"].as_str().unwrap().parse::<f32>().map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?,
                    department: value["kkyxmc"].as_str().unwrap().to_owned()
                },
                course_class: value["rwmc"].as_str().unwrap().to_owned(),
                course_type: value["kclbmc"].as_str().unwrap().to_owned(),
                id: value["id"].as_str().unwrap().to_owned(),
                major_teacher,
                major_time_and_place,
                minor_teacher,
                minor_time_and_place,
            },
            available: match value["sxbj"].as_str().unwrap() {
                "0" => { false },
                "1" => { true },
                _ => {false}
            },
            points: value["xkxs"].as_str().unwrap().parse::<u32>().unwrap(),
        };
        selected_courses_vec.push(course);
    }
    Ok(json::Json(selected_courses_vec))
}

#[rocket::get("/available_courses?<username>&<password>&<semester_year>&<semester_no>&<courses_type>")]
pub async fn available_courses(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    courses_type: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<Vec<AvailableCourse>>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
        let (major_teacher, 
            major_time_and_place, 
            minor_teacher, 
            minor_time_and_place) = parse_course_info(value["kcxx"].as_str().unwrap()).await;

        let course = AvailableCourse {
            advanced_course: AdvancedCourse {
                basic_course: Course {
                    course_id: value["kcdm"].as_str().unwrap().to_owned(),
                    course_name: value["kcmc"].as_str().unwrap().to_owned(),
                    credits: value["xf"].as_str().unwrap().parse::<f32>().map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?,
                    department: value["kkyxmc"].as_str().unwrap().to_owned(),
                },
                course_class: value["rwmc"].as_str().unwrap().to_owned(),
                course_type: value["kclbmc"].as_str().unwrap().to_owned(),
                id: value["id"].as_str().unwrap().to_owned(),
                major_teacher,
                major_time_and_place,
                minor_teacher,
                minor_time_and_place,
            },
            undergraduated_available: value["bksrl"].as_str().unwrap().parse::<u32>().unwrap(),
            undergraduated_selected: value["bksyxrlrs"].as_str().unwrap().parse::<u32>().unwrap(),
            graduated_available: value["yjsrl"].as_str().unwrap().parse::<u32>().unwrap(),
            graduated_selected: value["yjsyxrlrs"].as_str().unwrap().parse::<u32>().unwrap(),
            outline_id: value["kcid"].as_str().unwrap().to_owned(),
        };
        available_courses_vec.push(course);
    }
    Ok(json::Json(available_courses_vec))
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
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<serde_json::Value>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
    Ok(json::Json(v))
}

#[rocket::get("/drop_course?<username>&<password>&<semester_year>&<semester_no>&<course_id>")]
pub async fn drop_course(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    course_id: &str, 
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<serde_json::Value>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
    Ok(json::Json(v))
}

#[rocket::get("/update_points?<username>&<password>&<semester_year>&<semester_no>&<course_id>&<points>")]
pub async fn update_points(
    username: &str, 
    password: &str, 
    semester_year: &str, 
    semester_no: &str, 
    course_id: &str, 
    points: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<serde_json::Value>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

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
    Ok(json::Json(v))
}

#[rocket::get("/course_outline?<username>&<password>&<outline_id>")]
pub async fn course_outline(
    username: &str,
    password: &str,
    outline_id: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>
) -> Result<json::Json<serde_json::Value>, Unauthorized<String>> {

    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("kcid", outline_id);

    let v: serde_json::Value = client.post(OUTLINE_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    Ok(json::Json(v["content"]["kcdgbentity"]["kczwjj"].to_owned()))
}

#[rocket::get("/course_table?<username>&<password>&<semester_year>&<semester_no>")]
pub async fn course_table(
    username: &str,
    password: &str,
    semester_year: &str,
    semester_no: &str,
    client_storage: &State<Mutex<HashMap<String, Account>>>,
) -> Result<json::Json<Vec<CourseTableItem>>, Unauthorized<String>> {
    let tis_login_result = tis_login(username, password, client_storage).await?;
    if !tis_login_result { return Err(Unauthorized(None)); }

    let client_storage = client_storage.lock().await;
    let client = &client_storage.get(username).unwrap().client;

    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("bs", "2");
    post_form.insert("xn", semester_year);
    post_form.insert("xq", semester_no);

    let v: serde_json::Value = client.post(COURSE_TABLE_URL)
                                    .form(&post_form)
                                    .send()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                    .json::<serde_json::Value>()
                                    .await
                                    .map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    #[cfg(debug_assertions)]
    println!("{:?}", v);
    
    let mut course_table_items_vec = Vec::<CourseTableItem>::new();
    let json_array = v.as_array().unwrap();
    for item in json_array {
        let key = item["key"].as_str()
                            .unwrap();
        let day = key.chars().nth(2).unwrap().to_digit(10).unwrap();
        let time = key.chars().nth(6).unwrap().to_digit(10).unwrap();
        let course_table_item = CourseTableItem {
            day,
            time,
            course_info: item["kbxx"].as_str()
                                    .unwrap()
                                    .to_owned(),
        };
        course_table_items_vec.push(course_table_item);
    }
    Ok(json::Json(course_table_items_vec))
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