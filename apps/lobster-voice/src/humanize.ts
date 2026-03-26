import { humanizeNumbers } from "./numbers";

type DescriptionMap = Record<string, Record<string, string>>;

const STEP_DESCRIPTIONS: DescriptionMap = {
  // ─── pipeline.lobster.yaml ───
  pipeline: {
    "load-config": "Loading project configuration from cto-config.",
    "preflight": "Running preflight checks. Verifying API keys, bridge health, and cluster access.",
    "materialize-prd": "Materializing the PRD into the working directory.",
    "materialize-design-inputs": "Processing design inputs through the intake agent.",
    "save-design-bundle": "Saving the design bundle to the tasks directory.",
    "persist-design-metadata": "Persisting design metadata to Linear and Discord.",
    "generate-design-variants": "Generating design variants. Creating alternative visual directions using Stitch.",
    "design-review": "Posting design variants for your review. Check Discord or Linear to select your preferred design.",
    "save-design-selections": "Saving your design selections.",
    "notify-pipeline-start": "Notifying bridges that the pipeline has started.",
    "register-run": "Registering this pipeline run.",
    "setup-repo": "Setting up the GitHub repository.",
    "create-linear-project": "Creating the Linear project and syncing initial state.",
    "ensure-linear-session": "Ensuring the Linear agent session is active.",
    "build-infra-context": "Building infrastructure context. Scanning cluster, operators, and services.",
    "discover-tools": "Discovering available agent tools and MCP services.",
    "codebase-analysis": "Starting codebase analysis workflow.",
    "initial-parse-prd": "Running initial PRD parse. Decomposing requirements into tasks and identifying decision points.",
    "deliberation": "Starting the deliberation workflow. Optimist and Pessimist will debate the decision points.",
    "notify-task-gen": "Moving to task generation phase.",
    "intake": "Starting the main intake workflow. Generating tasks and artifacts.",
    "notify-pipeline-complete": "Pipeline complete. Intake handoff is ready.",
    "verify-intake-output": "Verifying that the intake workflow produced artifacts.",
    "deregister-run": "Deregistering the pipeline run.",
  },

  // ─── deliberation.lobster.yaml ───
  deliberation: {
    "research-prd": "Researching the PRD. Analyzing sources and extracting key requirements.",
    "init-debate": "Initializing the debate framework.",
    "notify-start": "Deliberation has begun.",
    "optimist-turn-1": "Round one. The Optimist is presenting their case.",
    "notify-opt-1": "Optimist round one complete. Notifying bridges.",
    "linear-activity-opt-1": "Recording Optimist round one in Linear.",
    "parse-dp-opt-1": "Parsing decision points from the Optimist's argument.",
    "pessimist-turn-1": "Round one. The Pessimist is presenting their counter-argument.",
    "notify-pes-1": "Pessimist round one complete. Notifying bridges.",
    "linear-activity-pes-1": "Recording Pessimist round one in Linear.",
    "parse-dp-pes-1": "Parsing decision points from the Pessimist's argument.",
    "resolve-dps-1": "Resolving decision points from round one.",
    "vote-dp-1": "Voting on round one decision points.",
    "verify-vote-1": "Verifying round one vote results.",
    "check-consensus-1": "Checking for consensus after round one.",
    "assemble-log-1": "Assembling the debate log for round one.",
    "resolved-after-1": "Round one decisions resolved.",
    "optimist-turn-2": "Round two. The Optimist responds.",
    "notify-opt-2": "Optimist round two complete.",
    "linear-activity-opt-2": "Recording Optimist round two in Linear.",
    "parse-dp-opt-2": "Parsing decision points from the Optimist's round two.",
    "pessimist-turn-2": "Round two. The Pessimist responds.",
    "notify-pes-2": "Pessimist round two complete.",
    "linear-activity-pes-2": "Recording Pessimist round two in Linear.",
    "parse-dp-pes-2": "Parsing decision points from the Pessimist's round two.",
    "resolve-dps-2": "Resolving decision points from round two.",
    "vote-dp-2": "Voting on round two decision points.",
    "verify-vote-2": "Verifying round two vote results.",
    "check-consensus-2": "Checking for consensus after round two.",
    "assemble-log-2": "Assembling the debate log for round two.",
    "resolved-after-2": "Round two decisions resolved.",
    "optimist-turn-3": "Final round. The Optimist makes their closing argument.",
    "notify-opt-3": "Optimist final round complete.",
    "linear-activity-opt-3": "Recording Optimist final round in Linear.",
    "parse-dp-opt-3": "Parsing decision points from the Optimist's closing.",
    "pessimist-turn-3": "Final round. The Pessimist makes their closing argument.",
    "notify-pes-3": "Pessimist final round complete.",
    "parse-dp-pes-3": "Parsing the Pessimist's final decision points.",
    "linear-activity-pes-3": "Recording Pessimist final round in Linear.",
    "collect-all-dps": "Collecting all decision points across all rounds.",
    "assemble-result": "Assembling the final deliberation result.",
    "notify-complete": "Deliberation complete. Notifying bridges.",
    "linear-activity-complete": "Recording deliberation completion in Linear.",
    "compile-brief": "Compiling the design brief from deliberation outcomes.",
    "save-brief": "Saving the design brief.",
  },

  // ─── intake.lobster.yaml ───
  intake: {
    "register-run": "Registering the intake run.",
    "linear-plan-parsing": "Updating Linear plan status to parsing.",
    "parse-prd": "Parsing the PRD. Extracting tasks, constraints, and requirements.",
    "verify-parse-prd": "Verifying the PRD parse output.",
    "breakpoint-parse-prd": "PRD parsing breakpoint. Checking quality.",
    "linear-activity-parse-prd": "Recording PRD parse results in Linear.",
    "analyze-complexity": "Analyzing task complexity and estimating effort.",
    "verify-analyze-complexity": "Verifying complexity analysis.",
    "breakpoint-analyze-complexity": "Complexity analysis breakpoint.",
    "linear-activity-analyze-complexity": "Recording complexity analysis in Linear.",
    "review-tasks": "Reviewing generated tasks for completeness.",
    "refine-tasks": "Refining tasks through the committee. Starting refinement rounds.",
    "verify-refine-tasks": "Verifying refined task output.",
    "breakpoint-refine-tasks": "Task refinement breakpoint.",
    "linear-activity-refine-tasks": "Recording task refinement in Linear.",
    "linear-plan-generation": "Updating Linear plan status to generation.",
    "generate-scaffolds": "Generating code scaffolds from refined tasks.",
    "verify-generate-scaffolds": "Verifying generated scaffolds.",
    "breakpoint-generate-scaffolds": "Scaffold generation breakpoint.",
    "linear-activity-generate-scaffolds": "Recording scaffold generation in Linear.",
    "fan-out-docs": "Fanning out documentation generation.",
    "validate-docs": "Validating generated documentation.",
    "write-docs": "Writing documentation files.",
    "linear-activity-write-docs": "Recording documentation in Linear.",
    "search-skills": "Searching for relevant agent skills.",
    "discover-skills": "Discovering skills that match the task requirements.",
    "verify-discover-skills": "Verifying discovered skills.",
    "linear-activity-discover-skills": "Recording skill discovery in Linear.",
    "generate-tool-manifest": "Generating the tool manifest.",
    "fan-out-prompts": "Fanning out prompt generation.",
    "validate-prompts": "Validating generated prompts.",
    "write-prompts": "Writing prompt files.",
    "linear-activity-write-prompts": "Recording prompt generation in Linear.",
    "generate-workflows": "Generating workflow definitions.",
    "validate-workflows": "Validating generated workflows.",
    "write-workflows": "Writing workflow files.",
    "generate-scale-tasks": "Generating scale and performance tasks.",
    "generate-security-report": "Generating the security assessment report.",
    "linear-activity-security-report": "Recording security report in Linear.",
    "generate-remediation-tasks": "Generating remediation tasks from the security report.",
    "verify-artifact-gates": "Verifying all artifact gates. Checking tasks, subtasks, decision points, docs, prompts, and workflows.",
    "verify-folder-structure": "Verifying the on-disk folder structure. Checking tasks, docs, prompts, and expected files.",
    "sync-linear-issues": "Syncing generated issues to Linear.",
    "linear-plan-commit": "Committing the Linear plan.",
    "commit-outputs": "Committing output artifacts to the repository.",
    "create-pr": "Creating the pull request.",
    "verify-delivery-gates": "Verifying delivery gates. Checking Linear sync and PR creation.",
    "linear-activity-pr-created": "Recording PR creation in Linear.",
    "write-handoff-summary": "Writing the handoff summary.",
    "deregister-run": "Deregistering the intake run. Intake complete.",
  },

  // ─── codebase-analysis.lobster.yaml ───
  "codebase-analysis": {
    "pack-repo": "Packing the repository for analysis.",
    "search-patterns": "Searching codebase for architectural patterns.",
    "summarize-codebase": "Summarizing the codebase structure with an LLM.",
    "save-context": "Saving codebase context to the tasks directory.",
  },

  // ─── task-refinement.lobster.yaml ───
  "task-refinement": {
    "expand-round-0": "Refinement round one. Expanding task definitions.",
    "vote-round-0": "Committee voting on refinement round one.",
    "check-round-0": "Checking refinement round one results.",
    "expand-round-1": "Refinement round two. Further expanding tasks.",
    "vote-round-1": "Committee voting on refinement round two.",
    "check-round-1": "Checking refinement round two results.",
    "expand-round-2": "Final refinement round. Polishing task definitions.",
    "vote-round-2": "Committee voting on the final refinement.",
    "check-round-2": "Checking final refinement results.",
    "resolve-output": "Resolving the best refinement output.",
  },

  // ─── voting.lobster.yaml ───
  voting: {
    "voter-1": "Architect is casting their vote.",
    "voter-2": "Pragmatist is casting their vote.",
    "voter-3": "Minimalist is casting their vote.",
    "voter-4": "Operator is casting their vote.",
    "voter-5": "Strategist is casting their vote.",
    "tally": "Tallying the committee votes.",
  },

  // ─── decision-voting.lobster.yaml ───
  "decision-voting": {
    "voter-1": "First voter is evaluating the decision point.",
    "voter-2": "Second voter is evaluating the decision point.",
    "voter-3": "Third voter is evaluating the decision point.",
    "voter-4": "Fourth voter is evaluating the decision point.",
    "voter-5": "Fifth voter is evaluating the decision point.",
    "tally": "Tallying decision point votes.",
    "verify-tally": "Verifying the vote tally.",
    "resolve": "Resolving the decision. Selecting the winner.",
    "publish-elicitation": "Publishing the decision for human review.",
  },
};

const GATE_DESCRIPTIONS: Record<string, string> = {
  "design-intake-agent": "Design intake agent check",
  "stitch-credentials": "Stitch API credentials check",
  "design-metadata-persistence": "Design metadata persistence check",
  "generate-design-variants": "Design variant generation gate",
  "design-review": "Design review feedback gate",
  "deliberation-retry": "Deliberation retry gate",
  "initial-parse-prd": "Initial PRD parse gate",
  "research-prd": "PRD research gate",
  "verify-vote-1": "Round one vote verification",
  "verify-vote-2": "Round two vote verification",
  "consensus-round-1": "Round one consensus check",
  "consensus-round-2": "Round two consensus check",
  "resolved-after-1": "Round one resolution",
  "resolved-after-2": "Round two resolution",
  "artifacts": "Artifact completeness gate",
  "delivery": "Delivery verification gate",
  "intake-output": "Intake output verification gate",
  "parse-prd": "PRD parsing gate",
  "verify-parse-prd": "PRD parse verification",
  "verify-refine-tasks": "Task refinement verification",
  "validate-docs": "Documentation validation gate",
  "validate-prompts": "Prompt validation gate",
  "folder-structure": "Folder structure verification gate",
  "verify-tally": "Vote tally verification",
  "resolve": "Decision resolution",
  "human-review": "Human review gate",
};

function describeResult(result: string): string {
  const lc = result.toLowerCase();
  if (lc === "passed") return "passed";
  if (lc.startsWith("failed")) return result.replace(/^failed:?\s*/i, "failed. ");
  if (lc.startsWith("retrying")) return "retrying";
  if (lc.startsWith("waiting")) return "waiting for input";
  if (lc.startsWith("winner")) return result;
  return result;
}

export function humanizeStep(workflow: string, step: string, context?: string): string {
  const desc = STEP_DESCRIPTIONS[workflow]?.[step];

  let message: string;
  if (desc) {
    message = desc;
  } else {
    const prettyStep = step.replace(/-/g, " ");
    const prettyWorkflow = workflow.replace(/-/g, " ");
    message = `${prettyWorkflow}: ${prettyStep}.`;
  }

  if (context) {
    message = `${message} ${humanizeNumbers(context)}`;
  }

  return humanizeNumbers(message);
}

export function humanizeGate(
  workflow: string,
  gate: string,
  result: string,
  context?: string,
): string {
  const gateLabel = GATE_DESCRIPTIONS[gate] ?? gate.replace(/-/g, " ");
  const resultDesc = describeResult(result);

  let message = `${gateLabel}: ${resultDesc}.`;

  if (context) {
    message = `${message} ${humanizeNumbers(context)}`;
  }

  return humanizeNumbers(message);
}

export function humanizeRaw(text: string): string {
  return humanizeNumbers(text);
}
