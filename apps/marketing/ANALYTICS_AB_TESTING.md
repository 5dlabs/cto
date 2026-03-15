# Hero Headline A/B Test Tracking (Marketing)

This landing page now runs a 3-variant hero headline experiment and tracks engagement events with the assigned variant.

## Variants

- `idea_impact_fast`: `From Idea to Impact, Fast`
- `idea_impact`: `From Idea to Impact`
- `concept_customer_value`: `From Concept to Customer Value`

Assignment is random on first visit and persisted per browser in `localStorage` with key:
- `cto_home_hero_headline_variant_v1`

## Tracked Events

- `hero_experiment_impression`
- `hero_start_now_click`
- `hero_waitlist_submit_attempt`
- `hero_waitlist_submit_success`
- `hero_waitlist_submit_error`

Each event includes:
- `experiment: "hero_headline_v1"`
- `variant: <one of the 3 ids above>`
- `location: "home_hero"`

## Env Vars

Set any analytics providers you use for the marketing app:

- `NEXT_PUBLIC_GA_MEASUREMENT_ID=G-XXXXXXXXXX`
- `NEXT_PUBLIC_UMAMI_WEBSITE_ID=<your-umami-website-id>`
- Optional: `NEXT_PUBLIC_UMAMI_SRC=https://cloud.umami.is/script.js`

If a provider is configured, events are forwarded automatically.

## Where To Get GA ID (Not an API Key)

You need a **Measurement ID** (format `G-...`), not an API key.

1. Open Google Analytics.
2. Go to **Admin** (gear icon).
3. In **Property**, open **Data Streams**.
4. Select your Web stream for `cto.5dlabs.ai`.
5. Copy **Measurement ID** (looks like `G-ABC123XYZ9`).
6. Put it in `NEXT_PUBLIC_GA_MEASUREMENT_ID`.

## Quick Reporting Setup (GA4 Explore)

Create a Free Form exploration with:

- Rows: `variant`
- Columns: `eventName` (or filter one event at a time)
- Metrics: `Event count`
- Filters:
  - `eventName` in `hero_experiment_impression`, `hero_waitlist_submit_success`
  - `experiment` equals `hero_headline_v1`

Use these as primary KPIs:

1. Exposure per variant: count of `hero_experiment_impression`
2. Conversion per variant: count of `hero_waitlist_submit_success`
3. Conversion rate per variant: `waitlist_success / impression`

Run the test until each variant has a meaningful sample before choosing a winner.
