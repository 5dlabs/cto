import type { ProcessedMessage } from "./types";

/**
 * Inject a processed NATS message into the agent's session.
 *
 * Uses OpenClaw runtime APIs:
 *   - enqueueSystemEvent: delivers text to the agent session
 *   - requestHeartbeatNow: wakes the agent immediately for urgent messages
 */
export function deliverToAgent(
  runtime: any,
  processed: ProcessedMessage,
  logger: { info: Function; warn: Function; error: Function },
): void {
  try {
    if (typeof runtime?.system?.enqueueSystemEvent === "function") {
      runtime.system.enqueueSystemEvent(processed.eventText, {
        sessionKey: processed.sessionKey,
      });
    } else {
      logger.warn(
        `enqueueSystemEvent not available — message from ${processed.raw.from} logged but not delivered to session`,
      );
      return;
    }

    if (processed.priority === "urgent") {
      if (typeof runtime?.system?.requestHeartbeatNow === "function") {
        runtime.system.requestHeartbeatNow({
          reason: `Urgent NATS message from ${processed.raw.from}`,
        });
      } else {
        logger.warn(
          "requestHeartbeatNow not available — urgent message delivered but agent not woken",
        );
      }
    }

    logger.info(
      `Delivered message from ${processed.raw.from} (priority=${processed.priority})`,
    );
  } catch (err) {
    logger.error(`Failed to deliver NATS message to agent: ${err}`);
  }
}
