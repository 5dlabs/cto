/**
 * design-review — Post design variants to both bridges and wait for selection.
 *
 * Reads design-variants.json, posts each screen's variants to Discord and
 * Linear via their /design-review endpoints, polls for selection, and writes
 * design-selections.json.
 */

import * as fs from 'fs';

interface DesignVariant {
  variant_id: string;
  source_screen_id: string;
  source_target: string;
  label: string;
  description: string;
  aspects_changed: string[];
  screen_id?: string;
  html_url?: string;
  image_url?: string;
  status: string;
  error?: string;
}

interface DesignVariantsResult {
  project_id: string;
  screens: Array<{
    source_screen_id: string;
    source_target: string;
    source_provider?: string;
    variants: DesignVariant[];
  }>;
  total_variants: number;
  timestamp: string;
}

interface DesignReviewRequest {
  review_id: string;
  session_id: string;
  screen_context: string;
  variants: Array<{
    variant_id: string;
    label: string;
    image_url: string;
    html_url?: string;
    description: string;
    aspects_changed: string[];
  }>;
  recommended_variant?: string;
  timeout_seconds: number;
  timestamp: string;
  discord_channel_id?: string;
  linear_issue_id?: string;
  metadata?: Record<string, string>;
}

interface ElicitationStatus {
  active: boolean;
  known: boolean;
}

interface DesignSelection {
  screen_id: string;
  screen_target: string;
  review_id: string;
  selected_variant_id?: string;
  selected_label?: string;
  response_type: 'selection' | 'request_changes' | 'timeout';
  user_notes?: string;
  source: 'discord' | 'linear' | 'timeout';
  timestamp: string;
}

async function postToBridge(url: string, payload: unknown): Promise<boolean> {
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    return res.ok;
  } catch {
    return false;
  }
}

async function readStatus(url: string): Promise<ElicitationStatus | undefined> {
  try {
    const res = await fetch(url);
    if (!res.ok) return undefined;
    const data = await res.json() as Partial<ElicitationStatus>;
    if (typeof data.active !== 'boolean' || typeof data.known !== 'boolean') return undefined;
    return { active: data.active, known: data.known };
  } catch {
    return undefined;
  }
}

async function waitForResolution(
  reviewId: string,
  discordUrl: string,
  linearUrl: string,
  posted: { discord: boolean; linear: boolean },
): Promise<void> {
  const targets: string[] = [];
  if (posted.discord) targets.push(`${discordUrl}/elicitation/status/${encodeURIComponent(reviewId)}`);
  if (posted.linear) targets.push(`${linearUrl}/elicitation/status/${encodeURIComponent(reviewId)}`);
  if (targets.length === 0) return;

  const timeoutSec = parseInt(
    process.env['DESIGN_REVIEW_TIMEOUT_SECONDS'] ?? process.env['ELICITATION_TIMEOUT_SECONDS'] ?? '1800',
    10,
  );
  const deadline = timeoutSec > 0 ? Date.now() + (timeoutSec + 30) * 1000 : 0;

  await new Promise(r => setTimeout(r, 1000));

  for (;;) {
    if (deadline > 0 && Date.now() > deadline) {
      console.error(`[design-review] Timeout waiting for ${reviewId}`);
      return;
    }
    const statuses = await Promise.all(targets.map(t => readStatus(t)));
    const known = statuses.filter((s): s is ElicitationStatus => !!s && s.known);
    if (known.length > 0 && known.every(s => !s.active)) return;
    await new Promise(r => setTimeout(r, 2000));
  }
}

export interface DesignReviewArgs {
  variantsPath: string;
  sessionId: string;
  outputPath: string;
  linearIssueId?: string;
  discordChannelId?: string;
  runId?: string;
}

export async function designReview(args: DesignReviewArgs): Promise<DesignSelection[]> {
  const discordUrl = process.env['DISCORD_BRIDGE_URL'] ?? 'http://discord-bridge.bots.svc:3200';
  const linearUrl = process.env['LINEAR_BRIDGE_URL'] ?? 'http://linear-bridge.bots.svc:3100';
  const timeoutSec = parseInt(
    process.env['DESIGN_REVIEW_TIMEOUT_SECONDS'] ?? process.env['ELICITATION_TIMEOUT_SECONDS'] ?? '1800',
    10,
  );

  const raw = fs.readFileSync(args.variantsPath, 'utf-8');
  const variantsResult: DesignVariantsResult = JSON.parse(raw);

  const selections: DesignSelection[] = [];

  for (const screen of variantsResult.screens) {
    const generatedVariants = screen.variants.filter(v => v.status === 'generated' && v.image_url);
    if (generatedVariants.length === 0) {
      console.error(`[design-review] No generated variants for ${screen.source_target} screen — skipping`);
      continue;
    }

    const reviewId = `${args.sessionId}-design-${screen.source_screen_id}`;
    const provider = screen.source_provider ?? 'stitch';
    const screenContext = `${screen.source_target}/${provider} (${screen.source_screen_id.slice(0, 8)})`;

    const request: DesignReviewRequest = {
      review_id: reviewId,
      session_id: args.sessionId,
      screen_context: screenContext,
      variants: generatedVariants.map(v => ({
        variant_id: v.variant_id,
        label: v.label,
        image_url: v.image_url!,
        html_url: v.html_url,
        description: v.description,
        aspects_changed: v.aspects_changed,
      })),
      recommended_variant: generatedVariants[0]?.variant_id,
      timeout_seconds: timeoutSec,
      timestamp: new Date().toISOString(),
      discord_channel_id: args.discordChannelId,
      linear_issue_id: args.linearIssueId,
      metadata: {
        source_provider: provider,
        ...(args.runId ? { run_id: args.runId } : {}),
      },
    };

    console.error(`[design-review] Posting ${generatedVariants.length} variants for ${screenContext}...`);

    const [discord, linear] = await Promise.all([
      postToBridge(`${discordUrl}/design-review`, request),
      postToBridge(`${linearUrl}/design-review`, request),
    ]);

    console.error(`[design-review] Posted: discord=${discord}, linear=${linear}`);

    if (discord || linear) {
      console.error(`[design-review] Waiting for selection on ${reviewId}...`);
      await waitForResolution(reviewId, discordUrl, linearUrl, { discord, linear });
      console.error(`[design-review] Selection received for ${reviewId}.`);
    }

    selections.push({
      screen_id: screen.source_screen_id,
      screen_target: screen.source_target,
      review_id: reviewId,
      selected_variant_id: generatedVariants[0]?.variant_id,
      selected_label: generatedVariants[0]?.label,
      response_type: 'selection',
      source: discord ? 'discord' : linear ? 'linear' : 'timeout',
      timestamp: new Date().toISOString(),
    });
  }

  fs.writeFileSync(args.outputPath, JSON.stringify(selections, null, 2));
  return selections;
}
