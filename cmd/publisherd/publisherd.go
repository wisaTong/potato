package main

import (
	"log"
	"net/http"
	"os"
	"text/template"
)

//Student something
type Student struct {
	Name      string
	College   string
	StudentID int
}

//Handler something
func handler(w http.ResponseWriter, r *http.Request) {
	student := Student{
		Name:      "John Doe",
		College:   "Kasetsart",
		StudentID: 601054000,
	}
	dir, _ := os.Getwd()
	t, _ := template.ParseFiles(dir + "/templates/test.html")
	err := t.Execute(w, student)
	if err != nil {
		log.Fatal(err)
	}

}

func main() {
	// Start publisherd
	http.HandleFunc("/", handler)
	http.ListenAndServe(":8000", nil)

}
