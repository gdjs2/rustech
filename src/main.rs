use std::{collections::HashMap};

use futures::lock::Mutex;
use rustech::{available_courses, basic_info, cas_login, courses_grades, get_courses, index, select_course, selected_courses, semester_gpa};

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
            .manage(Mutex::new(HashMap::<String, reqwest::Client>::new()))
            .mount("/", rocket::routes![index, 
                                                    cas_login, 
                                                    basic_info, 
                                                    semester_gpa,
                                                    courses_grades,
                                                    get_courses,
                                                    selected_courses,
                                                    available_courses,
                                                    select_course])
}
