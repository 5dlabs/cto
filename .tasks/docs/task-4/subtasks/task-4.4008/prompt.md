Implement subtask 4008: Implement tax calculation engine for GST/HST, US sales tax, and international

## Objective
Build a modular tax calculation service that computes applicable taxes based on jurisdiction, integrating into invoice creation and reporting.

## Steps
1. Create src/services/tax.rs with a TaxCalculator trait and implementations.
2. Define tax jurisdiction types: Canadian (GST/HST by province), US (state sales tax), International (VAT or exempt).
3. Implement CanadianTaxCalculator:
   - Lookup table: province → GST rate, HST rate (e.g., ON=13% HST, AB=5% GST, BC=5% GST+7% PST)
   - calculate_tax(subtotal, province) → returns tax breakdown (gst_amount, hst_amount or pst_amount, total_tax)
4. Implement USTaxCalculator:
   - Lookup table: state → sales tax rate (can be simplified to major states for v1)
   - calculate_tax(subtotal, state) → returns (sales_tax_amount, total_tax)
5. Implement InternationalTaxCalculator:
   - Default to 0% or configurable country-based VAT rate
6. Create a TaxService that determines jurisdiction from client address/location and dispatches to the correct calculator.
7. Integrate into invoice creation: when creating/updating an invoice, call TaxService to compute tax_amount based on client location.
8. Add tax_breakdown JSONB field to invoices (or extend existing tax_amount) to store itemized tax details.
9. Add migration if needed for tax_breakdown field.
10. Expose GET /v1/tax/calculate?subtotal=1000&jurisdiction=ON for ad-hoc tax calculation.

## Validation
Canadian tax: Ontario invoice at $1000 returns $130 HST; Alberta returns $50 GST; BC returns $50 GST + $70 PST. US tax: California at $1000 returns correct state sales tax. International: default 0% for unknown jurisdiction. Invoice creation with client in Ontario auto-calculates correct tax. Tax breakdown is stored and retrievable on invoice. Ad-hoc calculate endpoint returns correct amounts.