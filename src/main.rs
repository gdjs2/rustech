use std::{collections::HashMap};

use futures::lock::Mutex;
use rustech::apis::{available_courses, basic_info, cas_login, course_outline, courses_grades, current_semester, drop_course, get_courses, index, select_course, selected_courses, semester_gpa, update_points};
use rustech::structures::Account;

#[rocket::launch]
fn rocket() -> _ {
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
                                                    current_semester])
}
