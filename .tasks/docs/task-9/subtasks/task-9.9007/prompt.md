Implement subtask 9007: Build quote builder tab with Effect.Schema form validation and opportunity submission

## Objective
Implement app/(tabs)/quote.tsx as a ScrollView form. Accept pre-filled product params from router params. Render product list, event date fields, customer name/email/company inputs. Apply Effect.Schema validation on submit — show inline field-level error messages on decode failure. On valid submission call POST /api/v1/opportunities via useMutation. Show confirmation screen with quote_id on 201 success.

## Steps
Read `useLocalSearchParams` for pre-filled productId/startDate/endDate. Form state via useState or react-hook-form. On submit: `Schema.decodeUnknown(OpportunityDraft)(formValues)` — on ParseError, map field errors to inline messages below each input. On success: `mutation.mutateAsync(decoded)` calling `runEffect(createOpportunity(decoded))`. Show `<ConfirmationScreen quoteId={result.id} />` replacing form on success. ConfirmationScreen text: 'Your quote #${quoteId} has been submitted. Check Signal for Morgan's follow-up.' Validation rules: name required, email must match email regex via Schema.filter, company required, dates required.

## Validation
RNTL: submit empty form — assert error messages appear under each required field. Submit with invalid email — assert email error message. Submit valid form with msw returning 201 { id: 'opp_123' } — assert ConfirmationScreen renders with 'opp_123'. Submit with msw returning 500 — assert error banner shown, form not cleared.