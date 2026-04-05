Implement subtask 4005: Implement tax calculation engine for GST/HST, US sales tax, and international

## Objective
Build a configurable tax calculation engine that supports Canadian GST/HST, US state sales tax, and international tax rules, integrated into invoice creation.

## Steps
1. Create /src/services/tax.rs module. 2. Define a TaxCalculation struct: jurisdiction, tax_type (GST, HST, PST, US_SALES_TAX, VAT, EXEMPT), rate, calculated_amount. 3. Implement Canadian tax rules: GST (5%) applies federally; HST rates vary by province (e.g., ON 13%, NS 15%). Determine applicable rate based on customer province. 4. Implement US sales tax: rate varies by state. Store state tax rates in a configuration table or static map for v1. Apply based on customer state. 5. Implement international/VAT placeholder: apply a configurable VAT rate based on country. 6. Implement calculate_tax(line_items, customer_location) -> Vec<TaxCalculation> that determines applicable taxes and computes amounts. 7. Integrate into invoice creation: when creating/updating an invoice, call the tax engine and store tax breakdowns as part of invoice line items or a separate tax_details field. 8. Support tax-exempt status on customers.

## Validation
Canadian customer in Ontario gets 13% HST applied; Alberta customer gets 5% GST only; US customer in California gets correct state sales tax; tax-exempt customer gets zero tax; total tax on invoice matches sum of individual tax calculations; international customer gets VAT applied.