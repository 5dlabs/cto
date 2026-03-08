/**
 * classify-output — Map CLI output to Linear activity types.
 */

export type ActivityType = 'thought' | 'action' | 'elicitation' | 'response' | 'error';

export interface ActivityClassification {
  type: ActivityType;
  body: string;
  action?: string;
  parameter?: string;
  result?: string;
  ephemeral: boolean;
}

export function classifyCliOutput(
  output: string,
  cli: 'claude' | 'codex' | 'openclaw',
  isIntermediate: boolean,
): ActivityClassification {
  // Error detection (universal)
  if (/\b(error|Error|ERROR|exception|panic|fatal)\b/.test(output) && output.length < 2000) {
    return { type: 'error', body: output.slice(0, 4000), ephemeral: false };
  }

  if (cli === 'openclaw') {
    // Structured JSON from openclaw.invoke --action json
    try {
      const parsed = JSON.parse(output);
      if (parsed.tool_use || parsed.action) {
        return {
          type: 'action',
          body: output.slice(0, 4000),
          action: parsed.action ?? parsed.tool_use?.name ?? 'tool',
          parameter: parsed.tool_use?.input ? JSON.stringify(parsed.tool_use.input).slice(0, 500) : '',
          result: parsed.result?.slice(0, 500),
          ephemeral: isIntermediate,
        };
      }
      return {
        type: isIntermediate ? 'thought' : 'response',
        body: typeof parsed === 'string' ? parsed : JSON.stringify(parsed).slice(0, 4000),
        ephemeral: isIntermediate,
      };
    } catch {
      // Not JSON — treat as text
    }
  }

  if (cli === 'claude') {
    // Claude Code CLI: tool-use blocks marked with [tool_use] or similar
    if (/\[tool_use\]|\btool_use\b|Tool:/.test(output)) {
      const toolMatch = output.match(/Tool:\s*(\w+)/);
      return {
        type: 'action',
        body: output.slice(0, 4000),
        action: toolMatch?.[1] ?? 'tool',
        parameter: '',
        ephemeral: isIntermediate,
      };
    }
  }

  // Default: thought for intermediate, response for final
  return {
    type: isIntermediate ? 'thought' : 'response',
    body: output.slice(0, 4000),
    ephemeral: isIntermediate,
  };
}
