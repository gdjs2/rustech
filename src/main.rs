use std::{collections::HashMap};

use futures::lock::Mutex;
use rustech::apis::*;
use rustech::structures::Account;

#[rocket::launch]
fn rocket() -> _ {
    simple_logging::log_to_file("./log.txt", log::LevelFilter::Info)
                    .unwrap();
    rocket::build()
            .manage(Mutex::new(HashMap::<String, Account>::new()))
            .mount("/", rocket::routes![index, 
                                                    cas_login, 
                                                    basic_info, 
                                                    semester_gpa,
                                                    courses_grades,
                                                    get_courses,
                                                    selected_courses,
                                                    available_courses,
                                                    select_course,
                                                    drop_course,
                                                    update_points,
                                                    course_outline,
                                                    course_table])
}
