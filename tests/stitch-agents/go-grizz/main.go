// Stitch test fixture for Grizz (Go backend agent)
//
// This file contains intentional issues for testing remediation:
// - Error not checked
// - Unused imports
// - Missing documentation
package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
)

type User struct {
	ID    int64  `json:"id"`
	Name  string `json:"name"`
	Email string `json:"email"`
}

// TODO: Intentional issue - error not checked
func getUsers(w http.ResponseWriter, r *http.Request) {
	users := []User{
		{ID: 1, Name: "Alice", Email: "alice@example.com"},
		{ID: 2, Name: "Bob", Email: "bob@example.com"},
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(users) // error ignored
}

// TODO: Intentional issue - unused variable
func getUserByID(w http.ResponseWriter, r *http.Request) {
	userID := chi.URLParam(r, "id")
	unusedVar := "this is unused"

	user := User{ID: 1, Name: "Alice", Email: "alice@example.com"}

	fmt.Println(userID) // just to use it
	_ = unusedVar       // suppress unused error but still bad practice

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(user)
}

func main() {
	r := chi.NewRouter()
	r.Use(middleware.Logger)

	r.Get("/users", getUsers)
	r.Get("/users/{id}", getUserByID)

	port := os.Getenv("PORT")
	if port == "" {
		port = "3000"
	}

	http.ListenAndServe(":"+port, r) // error ignored
}
