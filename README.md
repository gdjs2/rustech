# Rustech
A Web API for SUSTech TIS written in RUST.

一个 SUSTech 教务系统 (TIS) 的 API 封装。
### Overview 综述

This is a project, which is a web service providing serveral simple re-encapsulation APIs of SUSTech TIS, including qeurying basic information, overall GPA, course grades of students. It has pretty JSON tag for query result and reduce jumbled information. And also, it is written in Rust, which may be safer than the traditional Java/Golang/C web services.

这是一个非常简陋的 SUSTech 新教务系统 (TIS) 的 API 封装，封装了基本的 CAS 登录、查询基本信息、GPA 和分科成绩的 API。它的查询结果跟 TIS 提供的接口相比拥有更好的信息识别度，并且去除了许多冗杂的返回结果。它是 100% 用 Rust 语言写成，安全性相比传统的 Java/Golang/C 后端更有保障。

### APIs 接口
All the APIs work with GET method with correct username (SID) and password for CAS. 

所有的 API 都是通过 GET 方法进行请求，查询参数都是对应的学校 CAS 系统用户名（学号）以及密码。
1. `/cas_login?username=&password=`: This is the API for you to test the validation of a CAS account. It will return a simple "Hello World!" if the CAS accouant can be used to login successfully, or 401 if you provide a invalid account information. 基本的测试 CAS 登录的接口，登录成功则返回简单的 "Hello World!" 信息，否则会返回 401 代码。
2. `/basic_info?username=&password=`: Query the basic information of the students, which includes TIS ID, SID, name, email, the year getting into the SUSTech, department and major. 查询学生的基本信息，包括 TIS ID、学号、姓名、邮箱、入学年份、部门以及专业。
3. `/semester_gpa?username=&password=`: Query the GPA in semester. This query will return a json object includes overall gpa, rank and an array of GPAs of each semester. 按学期查询 GPA，查询结果是一个 JSON 对象，包含了总体 GPA、排名以及一个存储了所有学期 GPA 的 JSON 数组。
4. `/courses_grades?username=&password=`: Query the grades of each course. This query will return a json array includes grade of each course. This API only query for the most recent 100 classes you finish as I have not found anyone could finish more than 100 courses during undergraduate. 按学科查询成绩，查询结果是一个 JSON 数组，包括了所有科目的成绩。因为目前还没有本科专业需要修超过 100 科课程，所以目前这个接口仅仅查询最近 100 科的成绩。
5. `/courses`: Get all the courses from TAO of SUSTech. 从本科生教育网上获取所有的本科生课程。
6. `/selected_courses?username=&password=&semester_year=&semester_no=`: Qeury the selected courses of the specific semester. In addition to the username and password, you should give extra two parameters semester_year and semester_no. semester_year is in the format like *2020-2021*, which means the semester year of Aug. 2020 to Jun. 2021. semester_no is integer from 1~3, which are corresponding to autumn, spring and summer semester year. A full query link may be like `/selected_courses?username=11810000&password=***&semester_year=2020-2021&semester_no=2` which means to query the selected courses in the spring semester of 2021. 查询特定学年的成绩，除去用户名和密码，还需要提供额外的两个参数，分别代表学年以及对应的学期。这里的学年以及学期的格式跟南科大教务系统上的保持一致，2020-2021 表示从 2020 年 8 月份开始，到 2021 年 6 月份结束的这个学年，1、2、3 分别代表了秋季学期、春季学期以及夏季学期。一个完整的查询例子是 `/select_courses?username=11810000&password=***&semester_year=2020-2021&semester_no=2`，代表查询 2021 年度春季学期该学生的所选课程。
7. `/available_courses?username=&password=&semester_year=&semester_no=&courses_type=`: Query the available courses of the specific semester. In addition to the parameters the same as upon, there is another parameter called `courses_type`. This parameter is corresponding to the tag on the top of tis system including "General Required", "General Elective" and so on. There are four choice for this parameter, which are "GR" for "General Required Classes", "GE" for "General Elective Classes", "TP" for "The Classes within Training Plan" and "NTP" for "The Classes without Training Plan". 查询特定学期的可选课程，除去和以上一点相同的学期信息以外，额外参数 `courses_type` 还需要提供查询的可选课程类别。该参数一共有四个选项，分别是 “GR” 对应通识必修课，“GE”对应通识选修课，“TP”对应培养方案内课程，“NTP”对应非培养方案内课程（这四个选项与 TIS 系统上方的四个标签相对应）。
8. `/select_course?username=&password=&semester_year=&semester_no=&course_id=&course_type=&points=`: Select the specific course. The `semester_year` and `semester_no` must be corresponding to the current course selection period. The `course_type` must be the correct one to the selected course or the selection will go wrong which will be hard to fix. The points is the points you want to use to select the course. 选取选定的课程。`semester_year` 和 `semester_no` 参数必须与当前开放选课的玄奇相匹配。`course_type` 参数必须要和你选定的课程的类型相匹配，不然会出现难以修复的问题。`points` 参数代表你选课所投入的分数。
9. `/drop_course?username=&password=&semester_year=&semester_no=&course_id=`: Drop out the specific class. The requirements for `semester_year` and `semester_no` are the as the one uppon. 退课，将会退掉选定的课程，参数 `semester_year` 和 `semester_no` 需要满足的要求和选课 API 一致。
10. `/update_points?username=&password=&semester_year=&semester_no=&course_id=&points=`: Update the points for one of your selected course. `points` is the points you want to choose for the specific course. 调整你所选某个科目的选课积分，`points` 参数代表你所想要调整到的积分。

### Maintainance 维护
This project will NOT be maintained regularly. So if you have good idea about refine it or the APIs of TIS has changed and you want to make it compatible to new system, PR is welcomed!!!

由于年级问题，这个项目不会被经常性地维护，所以如果你对如何改善它或者维护它有好的想法，欢迎跟我联系。如果你发现教务系统的接口更新了，你想更新这个项目，也欢迎提交 PR。

Also, as I am not familiar with the front-end development. I am looking forward to the one who can help me to build a corresponding **front-end system** (WeChat Mini Program is prefered). As we know, the web application of TIS system is not compatible to the mobile device. So if you are interested in making such a front-end system, you can contact me. I have **EXTREMELY STRONG CONNECTION** with ITS of SUSTech, I can recommend our system to the ITS which may become expansion of our TIS system.

同时，因为我不是很熟悉前端开发 （我的主要研究方向在系统安全），我在寻找可以帮助我写一个对应**前端**系统的同学（微信小程序优先）。众所周知，目前学校的教务系统还没有对移动端进行适配。如果你对这方面的开发感兴趣，非常欢迎联系我，我可以将我们的整个系统成品提交给学校信息中心，说不定能够成为学校教务系统的一个扩展（类似于校巴小程序）。

### TODO
There are a lot of work to do to improve this project:

目前这个系统还有很多地方可以改进：

- [ ] Build the database of cultivate scheme, and show the information of the cultivating situation of cuurent user. 维护一个培养方案的数据库，可以展示当前学生的培养方案情况，包括未修读课程等等。
- [X] Refine the data structure for the course grade. As I have no account which failed in any class, so the current data structure does not contains the grade information about failed courses. 改进储存课程成绩的结构体，因为我没有任何有过挂科情况课程，所以目前该数据结构还没有对这一块进行处理，初步感觉为了处理不同原因的挂科，教务系统在这里对分数是有特殊处理的。
- [ ] Refine the login logic. Now we are using a proxy-like way to login, which need the user to pass the password to our service. I prefer the front-end to redirect the user to the **OFFICIAL** CAS web page, and pass the ticket of the TIS system back to the web. 改进登录逻辑，目前我们使用的是代替用户进行 CAS 登录方案，但这个方案需要将用户的账号和密码发送到我们的后端，这在严谨的用户看来是非常不安全的行为（尽管学校很多私有系统都是采用的这种方案）。所以我想通过前端的重定向，将网页定向到学校官网的 CAS 登录页面，当用户登录完重定向至 TIS 的时候，前端能够将从 url 里面捕获到的 ticket 传递给后端，这样用户的登录账号以及密码就不用通过我们的后端，就能做到相对的安全。
- [x] Encapsulate the course-select system. 封装选课 API。
- [ ] Improve the code quality. 提高代码质量。（我是 Rust 新手 T.T）
- [x] Add the API to get all courses. 添加查询所有课程的接口。
- [x] Add the API to get selected courses. 添加查询所有已选课程的接口。
- [x] Add the API to get available courses. 添加查询所有可选课程的接口。

### Disclaimer 免责声明
All the TIS APIs which I use in this project can be found in https://tis.sustech/.edu.cn. During the test, I **ONLY** use the CAS account of mine. The APIs which are used in the project cannot be used illegally. All legal liability for unlawful calls to the TIS API is the responsibility of the caller. 

程序当中所用到的所有接口都是 TIS 的公开接口。在整个测试过程当中，我仅仅使用了我自己的 CAS 账号。所有在本程序里公开的 TIS 接口都不能被非法的调用，因非法调用所产生的一切法律责任由调用者自行承担。

### Contact Me 联系我
SUSTecher is strongly welcome to contact me to maintain this project. 

欢迎任何南科大的学生联系我，一起维护这个项目。

e-mail: xzqgdjs2@icloud.com