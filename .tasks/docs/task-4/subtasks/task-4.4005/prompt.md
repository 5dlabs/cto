Implement subtask 4005: Implement basic Stripe integration for payment recording

## Objective
Implement basic Stripe integration for recording payment events, focusing on capturing payment details rather than full processing.

## Steps
1. Integrate a Stripe client library to record payment events (not full processing).2. Implement a webhook handler to receive Stripe payment notifications and record them.

## Validation
Simulate a Stripe payment webhook and verify a payment is recorded in the database and linked to an invoice.