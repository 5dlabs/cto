/**
 * LemonSlice agent IDs — two separate agents, never mix them up.
 *
 * PRODUCT (teams) Morgan — agent_3adc6522f21cc204
 *   Used on /cto/morgan (LemonSliceWidget default). Talks to paying customers / teams.
 *   https://lemonslice.com/agents/agent_3adc6522f21cc204
 *
 * INVESTOR Morgan — agent_0b8ca791bd37c632
 *   Used on /pitch CTA ("Talk to Morgan"). Talks to investors.
 *   https://lemonslice.com/agents/agent_0b8ca791bd37c632
 */
export const PRODUCT_MORGAN_AGENT_ID = "agent_3adc6522f21cc204";
export const INVESTOR_MORGAN_AGENT_ID = "agent_0b8ca791bd37c632";

export const investorMorganUrl = `https://lemonslice.com/agents/${INVESTOR_MORGAN_AGENT_ID}`;
export const productMorganUrl = `https://lemonslice.com/agents/${PRODUCT_MORGAN_AGENT_ID}`;
