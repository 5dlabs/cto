// Test file for Stitch E2E review testing
// This file contains intentional issues for testing purposes

// Issue 1: Use of var instead of let/const
var deprecatedVariable = "this should use let or const";

// Issue 2: Missing error handling
async function fetchData(url) {
    const response = await fetch(url);
    const data = await response.json();
    return data;
}

// Issue 3: Magic numbers
function calculatePrice(quantity) {
    return quantity * 19.99; // Should use a named constant
}

// Issue 4: Console.log in production code
function logMessage(message) {
    console.log(message); // Should use proper logging
}

// Issue 5: Empty block statement
function emptyFunction() {
    // TODO: implement this function
}

export { fetchData, calculatePrice, logMessage, emptyFunction };
