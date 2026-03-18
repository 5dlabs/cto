#!/usr/bin/env node
/**
 * Generate trading agent variant values files from matrix.json.
 *
 * Reads infra/gitops/agents/trader/matrix.json, generates per-variant
 * directories with values.yaml files, and optionally updates the
 * ApplicationSet elements list.
 *
 * Usage:
 *   node scripts/generate-trader-values.js              # Dry run (show what would be generated)
 *   node scripts/generate-trader-values.js --write       # Actually write files
 *   node scripts/generate-trader-values.js --write --update-appset  # Also update ApplicationSet
 */

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { load as yamlLoad, dump as yamlDump } from "js-yaml";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const TRADER_DIR = join(ROOT, "infra/gitops/agents/trader");
const MATRIX_FILE = join(TRADER_DIR, "matrix.json");
const APPSET_FILE = join(ROOT, "infra/gitops/applications/workloads/trading-agents.yaml");

const args = process.argv.slice(2);
const WRITE = args.includes("--write");
const UPDATE_APPSET = args.includes("--update-appset");

// Topic mappings for info-arb
const TOPIC_MAP = {
  geopolitics: "Iran,Russia,Ukraine,China,Taiwan,NATO,sanctions,ceasefire,invasion",
  tech: "AI,OpenAI,Google,Apple,semiconductor,antitrust,SEC,regulation",
  crypto: "Bitcoin,Ethereum,Solana,DeFi,SEC,Coinbase,Binance,stablecoin",
  economics: "Fed,inflation,GDP,unemployment,tariffs,recession,interest-rate",
  war: "missile,airstrike,troops,ceasefire,invasion,nuclear,defense,military",
};

function generateInfoArbVariant(topicFocus, confidence, maxUsd, bankroll) {
  const name = `info-arb-${topicFocus}-c${Math.round(confidence * 100)}-m${maxUsd}`;
  const riskLabel = confidence <= 0.2 ? "Aggressive" : confidence <= 0.3 ? "Moderate" : "Conservative";

  return {
    name,
    values: {
      namespace: "trading",
      agent: {
        id: `trader-${name}`,
        name: `Trader: Info Arb (${topicFocus}, ${riskLabel})`,
        model: "anthropic/claude-haiku-4-5-20251001",
        heartbeat: { every: "2m", prompt: "Run the info-arb scan pipeline. See HEARTBEAT.md." },
        sandbox: "off",
      },
      extraEnv: [
        { name: "INFO_ARB_MIN_CONFIDENCE", value: String(confidence) },
        { name: "INFO_ARB_MAX_USD", value: String(maxUsd) },
        { name: "INFO_ARB_BANKROLL", value: String(bankroll) },
        { name: "INFO_ARB_TOPICS", value: TOPIC_MAP[topicFocus] || topicFocus },
        { name: "STRATEGY_ID", value: name },
        { name: "GROK_API_KEY", valueFrom: { secretKeyRef: { name: "trader-api-keys", key: "grok-api-key" } } },
        { name: "SIMMER_API_KEY", valueFrom: { secretKeyRef: { name: "trader-api-keys", key: "simmer-api-key" } } },
        { name: "GEMINI_API_KEY", valueFrom: { secretKeyRef: { name: "trader-api-keys", key: "gemini-api-key" } } },
      ],
    },
  };
}

function generateTemporalVariant(threshold, maxUsd, kelly) {
  const name = `temporal-t${Math.round(threshold * 100)}-m${maxUsd}-k${Math.round(kelly * 100)}`;

  return {
    name,
    values: {
      namespace: "trading",
      agent: {
        id: `trader-${name}`,
        name: `Trader: Temporal (${threshold}%, $${maxUsd}, K${kelly})`,
        model: "anthropic/claude-haiku-4-5-20251001",
        heartbeat: { every: "5m", prompt: "Check temporal-arb process health." },
        sandbox: "off",
      },
      extraEnv: [
        { name: "TEMPORAL_ARB_MOMENTUM_THRESHOLD", value: String(threshold) },
        { name: "TEMPORAL_ARB_MAX_POSITION", value: String(maxUsd) },
        { name: "TEMPORAL_ARB_KELLY_CAP", value: String(kelly) },
        { name: "STRATEGY_ID", value: name },
        { name: "SIMMER_API_KEY", valueFrom: { secretKeyRef: { name: "trader-api-keys", key: "simmer-api-key" } } },
      ],
    },
  };
}

function generateMmVariant(spread, quoteSize, maxInventory) {
  const name = `mm-s${Math.round(spread * 100)}-q${quoteSize}-i${maxInventory}`;

  return {
    name,
    values: {
      namespace: "trading",
      agent: {
        id: `trader-${name}`,
        name: `Trader: MM (spread ${spread}, size ${quoteSize})`,
        model: "anthropic/claude-haiku-4-5-20251001",
        heartbeat: { every: "5m", prompt: "Check market-maker process health." },
        sandbox: "off",
      },
      extraEnv: [
        { name: "MM_SPREAD", value: String(spread) },
        { name: "MM_QUOTE_SIZE", value: String(quoteSize) },
        { name: "MM_MAX_INVENTORY", value: String(maxInventory) },
        { name: "STRATEGY_ID", value: name },
        { name: "SIMMER_API_KEY", valueFrom: { secretKeyRef: { name: "trader-api-keys", key: "simmer-api-key" } } },
      ],
    },
  };
}

// --- Main ---

const matrix = JSON.parse(readFileSync(MATRIX_FILE, "utf-8"));
const allVariants = [];

// Curated info-arb variants (not full cartesian)
const curatedInfoArb = [
  ["geopolitics", 0.15, 50, 500],
  ["geopolitics", 0.25, 25, 250],
  ["tech", 0.35, 10, 100],
  ["tech", 0.25, 25, 250],
  ["crypto", 0.25, 25, 250],
  ["economics", 0.25, 25, 250],
  ["war", 0.15, 50, 500],
];
for (const [topic, conf, maxUsd, bankroll] of curatedInfoArb) {
  allVariants.push(generateInfoArbVariant(topic, conf, maxUsd, bankroll));
}

// Curated temporal variants
const curatedTemporal = [
  [0.02, 5, 0.25],
  [0.03, 5, 0.25],
  [0.05, 10, 0.40],
  [0.08, 10, 0.40],
];
for (const [threshold, maxUsd, kelly] of curatedTemporal) {
  allVariants.push(generateTemporalVariant(threshold, maxUsd, kelly));
}

// Curated MM variants
const curatedMm = [
  [0.03, 1.0, 5],
  [0.04, 2.0, 10],
  [0.06, 5.0, 20],
];
for (const [spread, quoteSize, maxInv] of curatedMm) {
  allVariants.push(generateMmVariant(spread, quoteSize, maxInv));
}

console.log(`Generated ${allVariants.length} variant definitions:`);
for (const v of allVariants) {
  console.log(`  ${v.name}`);
}

if (WRITE) {
  for (const v of allVariants) {
    const dir = join(TRADER_DIR, v.name);
    if (!existsSync(dir)) mkdirSync(dir, { recursive: true });

    // Read base values and merge
    const strategyType = v.name.split("-")[0] === "info" ? "info-arb"
      : v.name.startsWith("temporal") ? "temporal-arb"
      : v.name.startsWith("mm") ? "mm"
      : null;

    if (!strategyType) {
      console.warn(`  Skipping ${v.name} — unknown strategy type`);
      continue;
    }

    const baseFile = join(TRADER_DIR, matrix[strategyType].base);
    const base = yamlLoad(readFileSync(baseFile, "utf-8"));

    // Deep merge variant values onto base
    const merged = { ...base, ...v.values };
    merged.agent = { ...base.agent, ...v.values.agent };

    const yaml = yamlDump(merged, { lineWidth: 120, noRefs: true });
    const outFile = join(dir, "values.yaml");
    writeFileSync(outFile, `# Auto-generated by scripts/generate-trader-values.js\n# Do not edit manually — re-run the generator instead.\n---\n${yaml}`);
    console.log(`  Wrote ${outFile}`);
  }

  if (UPDATE_APPSET) {
    // Read current ApplicationSet and update elements list
    const appsetContent = readFileSync(APPSET_FILE, "utf-8");
    const appset = yamlLoad(appsetContent);

    // Merge existing hand-written variants with generated ones
    const existingNames = new Set(
      appset.spec.generators[0].list.elements.map((e) => e.name)
    );
    for (const v of allVariants) {
      if (!existingNames.has(v.name)) {
        appset.spec.generators[0].list.elements.push({ name: v.name });
      }
    }

    writeFileSync(APPSET_FILE, yamlDump(appset, { lineWidth: 120, noRefs: true }));
    console.log(`  Updated ApplicationSet with ${allVariants.length} variants`);
  }
} else {
  console.log("\nDry run — use --write to create files, --update-appset to update ApplicationSet.");
}
