Implement subtask 4003: Define data models and repository layer for finance entities

## Objective
Implement Rust structs for all finance domain entities (Invoice, Payment, PayrollEntry, CurrencyRate) with serde and sqlx derives, plus repository functions for database CRUD operations.

## Steps
1. In src/models/invoice.rs: Define Invoice, InvoiceLine, CreateInvoiceRequest, UpdateInvoiceRequest structs with serde Serialize/Deserialize and sqlx::FromRow derives. Define InvoiceStatus enum with sqlx::Type. 2. In src/models/payment.rs: Define Payment, CreatePaymentRequest structs and PaymentStatus, PaymentMethod enums. 3. In src/models/payroll.rs: Define PayrollEntry, CreatePayrollRequest and PayrollStatus enum. 4. In src/models/currency.rs: Define CurrencyRate struct. 5. In src/services/invoice_repo.rs: Implement async functions: create_invoice (with line items in a transaction), get_invoice_by_id, list_invoices (with pagination, status filter), update_invoice_status, void_invoice. 6. In src/services/payment_repo.rs: create_payment, get_payments_for_invoice, update_payment_status. 7. In src/services/payroll_repo.rs: create_payroll_entry, list_payroll_entries (filter by period, status), approve_payroll, mark_payroll_paid. 8. In src/services/currency_repo.rs: upsert_rate, get_latest_rate, get_all_latest_rates. 9. All repository functions use sqlx::PgPool and return Result<T, AppError>.

## Validation
Unit tests verify struct serialization/deserialization; repository integration tests using sqlx::test with a real PostgreSQL instance verify CRUD operations for each entity; create_invoice correctly inserts invoice + line items in a single transaction.