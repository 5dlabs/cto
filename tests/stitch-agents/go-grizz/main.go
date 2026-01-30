package main

import (
	"fmt"
	"os"
)

// Code smells for Grizz to find:
// - error ignored
// - inefficient string building
// - unused variable

func main() {
	unusedVar := "hello"

	// Error ignored
	data, _ := os.ReadFile("config.json")
	fmt.Println(string(data))

	// Inefficient string concatenation in loop
	result := ""
	for i := 0; i < 100; i++ {
		result = result + fmt.Sprintf("%d, ", i)
	}
	fmt.Println(result)
}

func unusedFunction() {
	// TODO: implement
	x := 5
	y := 10
	fmt.Println(x)
}
