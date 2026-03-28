Implement subtask 4006: Implement RMS integration for quote-to-invoice conversion

## Objective
Implement logic for quote-to-invoice conversion, requiring integration with the RMS service (Task 3) to fetch project details.

## Steps
1. Implement a client for the RMS service (Task 3) to fetch project details based on a project ID.2. Develop an endpoint or internal function to trigger invoice creation from fetched RMS project data.

## Validation
Create a mock project in RMS, then trigger quote-to-invoice conversion in Finance and verify a new invoice is created with correct project details.