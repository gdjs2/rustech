use rocket::serde::Serialize;

#[derive(Serialize)]
pub struct BasicInfo {
    pub id: String,
    pub sid: String,
    pub name: String,
    pub email: String,
    pub year: String,
    pub department: String,
    pub major: String
}

#[derive(Serialize)]
pub struct SemesterGPA {
    pub semester_full_name: String,
    pub semester_year: String,
    pub semester_number: String,
    pub gpa: Option<f64>
}

#[derive(Serialize)]
pub struct StudentGPA {
    pub all_gpa: std::vec::Vec<SemesterGPA>,
    pub average_gpa: f64,
    pub rank: String
}

#[derive(Serialize)]
pub struct CourseGrade {
    pub code: String,
    pub name: String,
    pub class_hour: String,
    pub credit: u64,
    pub semester: String,
    pub final_grade: String,
    pub final_level: String,
    pub department: String,
    pub course_type: String
}

#[derive(Serialize)]
pub struct Course {
    pub course_id: String,
    pub course_name: String,
    pub credits: f32,
    pub department: String,
}

#[derive(Serialize)]
pub struct SelectedCourse {
    pub basic_course: Course,
    pub course_type: String,
    pub course_class: String,
    pub teacher: String,
    pub time_and_place: String,
    pub available: bool,
    pub id: String,
    pub points: u32,
}

#[derive(Serialize)]
pub struct AvailableCourse {
    pub basic_course: Course,
    pub course_type: String,
    pub course_class: String,
    pub teacher: String,
    pub time_and_place: String,
    pub id: String,
    pub undergraduated_available: u32,
    pub undergraduated_selected: u32,
    pub graduated_available: u32,
    pub graduated_selected: u32,
    pub outline_id: String,
}

pub struct Account {
    pub hash_salt: Option<(
        [u8; super::encrypt::CREDENTIAL_LEN], 
        [u8; super::encrypt::CREDENTIAL_LEN])>,
    pub client: reqwest::Client,
}