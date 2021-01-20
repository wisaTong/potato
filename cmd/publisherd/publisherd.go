package main

import (
	"html/template"
	"log"
	"net/http"
	"os"
)

//Student something
type Student struct {
	Name      string
	College   string
	StudentID int
}

func getTemplate() *template.Template {
	dir, _ := os.Getwd()
	templates := template.Must(template.ParseFiles(dir+"/templates/test.html", dir+"/templates/index.html"))
	return templates
}

//Handler something
func testHandler(w http.ResponseWriter, r *http.Request) {
	student := Student{
		Name:      "John Doe",
		College:   "Kasetsart",
		StudentID: 601054000,
	}
	err := getTemplate().ExecuteTemplate(w, "test.html", student)
	if err != nil {
		log.Fatal(err)
	}

}

func indexHandler(w http.ResponseWriter, r *http.Request) {
	err := getTemplate().ExecuteTemplate(w, "index.html", "")
	if err != nil {
		log.Fatal(err)
	}
}

func main() {
	// Start publisherd
	http.HandleFunc("/", testHandler)
	http.HandleFunc("/index", indexHandler)
	http.ListenAndServe(":8000", nil)

}
