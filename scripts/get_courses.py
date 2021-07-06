import requests
from bs4 import BeautifulSoup
# from elasticsearch import Elasticsearch

COURSES_URL = "https://course-tao.sustech.edu.cn/kcxxweb/KcxxwebChinesePC"
r = requests.get(COURSES_URL)
soup = BeautifulSoup(r.text, "lxml")
# es = Elasticsearch()

tables = soup.find_all('table')

head_table = tables[0]
departments_options = head_table.find_all("option")
department_list = {}
for option in departments_options:
	department_list["department_code"] = option.attrs['value']
	department_list["department_name"] = str.strip(str(option.string))
	print(department_list)
	# es.index(index="sustech_departments", doc_type="department", body=department_list)
	
tables.remove(tables[0])
course_list = {}
for table in tables:
	trs = table.find_all('tr')
	trs.remove(trs[0])
	for tr in trs:
		tds = tr.find_all('td')
		course_list["course_id"] = str(tds[0].find('a').string)
		course_list["course_name"] = str(tds[1].find('a').string)
		course_list["credits"] = float(tds[2].string)
		course_list["department"] = str(tds[4].string)
		# es.index(index="sustech_courses", doc_type="course", body=course_list)
		print(course_list)
		exit(0)

print("OK")