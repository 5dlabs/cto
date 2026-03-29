/**
 * Cloud vs. bare-metal cost framing for the deck (Mar 2026).
 * Not financial advice — stack-, region-, and contract-dependent.
 *
 * Sources (recheck before major fundraise):
 * - AWS data transfer out: https://aws.amazon.com/ec2/pricing/on-demand/ (Data Transfer section; tiered $/GB by region)
 * - Latitude.sh bandwidth: https://www.latitude.sh/network/pricing (20TB/mo included on Metal; US/UK/NL overage ≈ $1.25/TB)
 */

/** Mid-band illustrative multiple for internet egress $/GB: AWS public list vs Latitude US overage after bundle. */
export const EGRESS_MULTIPLE_ORDER_OF_MAGNITUDE =
  "Roughly 30–70× higher $/GB on marginal AWS internet egress vs. Latitude US overage (~$0.09 vs ~$0.00125/GB) — before counting 20TB free on bare metal.";

/** Example: 20TB/mo to internet — order-of-magnitude AWS egress bill vs $0 incremental on bundle-inclusive bare metal. */
export function illustrativeTwentyTbEgressMonthlyMaxUsd(): {
  awsEgressUsdApprox: number;
  bareMetalIncrementalUsd: number;
} {
  const gb = 20_000;
  const awsPerGbConservative = 0.085;
  return {
    awsEgressUsdApprox: Math.round(gb * awsPerGbConservative),
    bareMetalIncrementalUsd: 0,
  };
}

/** Exported for deck footnotes (single source for the ~20TB/mo story). */
export const ILLUSTRATIVE_AWS_EGRESS_20TB_USD_PER_MONTH =
  illustrativeTwentyTbEgressMonthlyMaxUsd().awsEgressUsdApprox;

/**
 * From internal notes (full cloud → bare-metal migrations, incl. managed markups).
 * Deck headline uses a slightly wider conservative band for VC read.
 */
export const INTERNAL_FULL_STACK_SAVINGS_RANGE_NOTES = "60–80%";

/** Deck headline — conservative range for “full stack” (compute + egress + managed markups), pending customer-specific model. */
export const DECK_INFRA_SAVINGS_LABEL = "50–75%";
export const DECK_REVENUE_STREAMS_COUNT = 4;
