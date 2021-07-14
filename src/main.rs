

#[rocket::get("/cas_login?<username>&<password>")]
async fn cas_login(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let _client = login(username, password, None).await?;
    // println!("{:?}", client);
    Ok("Hello world!".to_owned())
}

// #[rocket::get("/tis_login?<username>&<password>")]
async fn tis_login(username: &str, password: &str) -> Result<reqwest::Client, Unauthorized<String>> {
    let mut client = reqwest::Client::builder().cookie_store(true).gzip(true).user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0").build().map_err(|_| Unauthorized(Some("Unable to build the client".to_owned())))?;
    let tis_cas_url = "https://cas.sustech.edu.cn/cas/login?service=https://tis.sustech.edu.cn/cas";
    client = login(username, password, Some(client)).await.map_err(|_| Unauthorized(Some("Unable to login".to_owned())))?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Referer", reqwest::header::HeaderValue::from_static("https://tis.sustech.edu.cn/"));
    headers.insert("Accept", reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert("Accept-Language", reqwest::header::HeaderValue::from_static("zh-cn"));
    headers.insert("Accept-Encoding", reqwest::header::HeaderValue::from_static("gzip, deflate, br"));
    // headers.insert("RoleCode", reqwest::header::HeaderValue::from_static("01"));
    // headers.insert("Host", reqwest::header::HeaderValue::from_static("tis.sustech.edu.cn"));

    client.get(tis_cas_url).headers(headers).send()
            .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    // client.get("https://tis.sustech.edu.cn/user/basic").send().await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    Ok(client)
}

#[rocket::get("/basic_info?<username>&<password>")]
async fn basic_info(username: &str, password: &str) -> Result<String, Unauthorized<String>> {

    let client = tis_login(username, password).await?;
    const BASIC_INFO_URL: &str = "https://tis.sustech.edu.cn/UserManager/queryxsxx";
    let v: serde_json::Value = client.post(BASIC_INFO_URL).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                                        .json::<serde_json::Value>()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
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
async fn semester_gpa(username: &str, password: &str) -> Result<String, Unauthorized<String>> {

    let client = tis_login(username, password).await?;
    const SEMESTER_GPA_URL: &str = "https://tis.sustech.edu.cn/cjgl/xscjgl/xsgrcjcx/queryXnAndXqXfj";
    let v: serde_json::Value = client.post(SEMESTER_GPA_URL).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                                        .json::<serde_json::Value>()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
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
async fn courses_grades(username: &str, password: &str) -> Result<String, Unauthorized<String>> {
    let client = tis_login(username, password).await?;
    const COURSE_GRADES_URL: &str = "https://tis.sustech.edu.cn/cjgl/grcjcx/grcjcx";
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));
    let body = r#"{"xn":null,"xq":null,"kcmc":null,"cxbj":"-1","pylx":"1","current":1,"pageSize":100}"#;
    let v: serde_json::Value = client.post(COURSE_GRADES_URL).headers(headers).body(body).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                                        .json::<serde_json::Value>()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
    
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
    println!("Total {} course grades item", course_grades_vec.len());

    Ok(serde_json::to_string_pretty(&course_grades_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/courses")]
async fn get_courses() -> Result<String, Unauthorized<String>> {
    const COURSES_URL: &str = "https://course-tao.sustech.edu.cn/kcxxweb/KcxxwebChinesePC";
    let courses_html = reqwest::get(COURSES_URL).await.map_err(|_| Unauthorized(Some("Unable to get courses from the web".to_owned())))?
                                                .text().await.map_err(|_| Unauthorized(Some("Unable to get courses from the web".to_owned())))?;

    let cas_fragment = scraper::Html::parse_fragment(&courses_html[..]);
    let table_selector = scraper::Selector::parse("table").map_err(|_| Unauthorized(Some(String::from("Unable to parse HTML to fragment"))))?;
                    
    let mut table_iter = cas_fragment.select(&table_selector);
    let _head_table = table_iter.next().unwrap();
    // let option_selector = scraper::Selector::parse("option").map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    // let option_iter = head_table.select(&option_selector);
    let tr_selector = scraper::Selector::parse("tr").map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let td_selector = scraper::Selector::parse("td").map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let a_selector = scraper::Selector::parse("a").map_err(|_| Unauthorized(Some("Unable to parse the option selector".to_owned())))?;
    let mut courses_vec = Vec::<Course>::new();
    for table in table_iter {
        let mut tr_iter = table.select(&tr_selector);
        tr_iter.next();
        for tr in tr_iter {
            // println!("{}", tr.inner_html());
            let mut td_iter = tr.select(&td_selector);
            let course = Course {
                course_id: td_iter.next().unwrap().select(&a_selector).next().unwrap().inner_html(),
                course_name: td_iter.next().unwrap().select(&a_selector).next().unwrap().inner_html(),
                credits: td_iter.next().unwrap().inner_html().parse::<f32>().unwrap(),
                department: td_iter.next().unwrap().inner_html()
            };
            courses_vec.push(course);
        }
    }
    Ok(serde_json::to_string_pretty(&courses_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/selected_courses?<username>&<password>&<semester_year>&<semester_no>")]
async fn selected_courses(username: &str, password: &str, semester_year: &str, semester_no: &str) -> Result<String, Unauthorized<String>> {
    let client = tis_login(username, password).await?;
    const SELECTED_COURSES_URL: &str = "https://tis.sustech.edu.cn/Xsxk/queryYxkc";
    let mut post_form = std::collections::HashMap::<&str, &str>::new();
    post_form.insert("p_xkfsdm", "yixuan");
    post_form.insert("p_xn", semester_year);
    post_form.insert("p_xq", semester_no);
    let v: serde_json::Value = client.post(SELECTED_COURSES_URL).form(&post_form).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                                        .json::<serde_json::Value>()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;
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
            }
        };
        selected_courses_vec.push(course);
    }
    Ok(serde_json::to_string_pretty(&selected_courses_vec).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?)
}

#[rocket::get("/available_courses?<username>&<password>&<semester_year>&<semester_no>&<courses_type>")]
async fn available_courses(username: &str, password: &str, semester_year: &str, semester_no: &str, courses_type: &str) -> Result<String, Unauthorized<String>> {
    let client = tis_login(username, password).await?;
    const AVAILABLE_COURSES_URL: &str = "https://tis.sustech.edu.cn/Xsxk/queryKxrw";
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
    let v: serde_json::Value = client.post(AVAILABLE_COURSES_URL).form(&post_form).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
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

// Not work well. Fix by adding status in the server.
#[rocket::get("/select_course?<username>&<password>&<semester_year>&<semester_no>&<course_id>&<course_type>&<points>")]
async fn select_course(username: &str, password: &str, semester_year: &str, semester_no: &str, course_id: &str, course_type: &str, points: &str) -> Result<String, Unauthorized<String>> {
    let client = tis_login(username, password).await?;
    const SELECT_COURSE_URL: &str = "https://tis.sustech.edu.cn/Xsxk/addGouwuche"; // WTF???? 购物车？？？
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

    let v: serde_json::Value = client.post(SELECT_COURSE_URL).form(&post_form).send()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?
                                                        .json::<serde_json::Value>()
                                                        .await.map_err(|_| Unauthorized(Some("Unable to send the login redirect request to CAS".to_owned())))?;

    println!("{}", serde_json::to_string_pretty(&v).map_err(|_| Unauthorized(Some("Unable to parse the result to JSON".to_owned())))?);
    if v["gjhczztm"].as_str().unwrap() == "OPERATE.RESULT_SUCCESS" {
        return Ok("SUCCESS".to_owned());
    } else {
        return Err(Unauthorized(Some(v["message"].as_str().unwrap().to_owned())));
    }
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build().mount("/", rocket::routes!(index))
                    .mount("/", rocket::routes!(cas_login))
                    // .mount("/", rocket::routes!(tis_login))
                    .mount("/", rocket::routes!(basic_info))
                    .mount("/", rocket::routes!(semester_gpa))
                    .mount("/", rocket::routes!(courses_grades))
                    .mount("/", rocket::routes!(get_courses))
                    .mount("/", rocket::routes!(selected_courses))
                    .mount("/", rocket::routes!(available_courses))
                    .mount("/", rocket::routes!(select_course))
}
