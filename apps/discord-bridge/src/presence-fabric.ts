import { connect, StringCodec, type NatsConnection } from "nats";
import type { PresenceInbound, PresenceOutbound, PresenceRoute } from "./presence-types.js";

export interface PresenceFabric {
  publishRoute(route: PresenceRoute): void;
  publishInbound(event: PresenceInbound, route: PresenceRoute): void;
  publishOutbound(intent: PresenceOutbound): void;
  close(): Promise<void>;
}

const sc = StringCodec();

function safeSegment(value: string | undefined, fallback: string): string {
  return (value ?? fallback).toLowerCase().replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "") || fallback;
}

export function presenceInboundSubject(event: PresenceInbound, route: PresenceRoute): string {
  return [
    "cto",
    "presence",
    "in",
    safeSegment(event.runtime, "runtime"),
    safeSegment(event.coderun_id ?? route.coderun_id, "coderun"),
  ].join(".");
}

export function presenceOutboundSubject(intent: PresenceOutbound): string {
  const channelId = "target" in intent && intent.target ? intent.target.channel_id : "status";
  return ["cto", "presence", "out", safeSegment(intent.op, "op"), safeSegment(channelId, "channel")].join(".");
}

export function presenceRouteSubject(route: PresenceRoute): string {
  return [
    "cto",
    "presence",
    "route",
    safeSegment(route.runtime, "runtime"),
    safeSegment(route.agent_id, "agent"),
  ].join(".");
}

export async function createPresenceFabric(
  natsUrl: string | undefined,
  logger: { info: Function; warn: Function; error: Function },
): Promise<PresenceFabric> {
  if (!natsUrl) {
    return {
      publishRoute() {},
      publishInbound() {},
      publishOutbound() {},
      async close() {},
    };
  }

  let nc: NatsConnection | undefined;
  try {
    nc = await connect({ servers: natsUrl, name: "cto-presence-router" });
    logger.info(`Presence fabric connected to NATS at ${natsUrl}`);
  } catch (err) {
    logger.warn(`Presence fabric disabled; failed to connect to NATS at ${natsUrl}: ${err}`);
  }

  function publish(subject: string, payload: unknown): void {
    if (!nc || nc.isClosed()) return;
    nc.publish(subject, sc.encode(JSON.stringify(payload)));
  }

  return {
    publishRoute(route): void {
      publish(presenceRouteSubject(route), { kind: "route", route });
    },
    publishInbound(event, route): void {
      publish(presenceInboundSubject(event, route), { kind: "inbound", route_id: route.route_id, event });
    },
    publishOutbound(intent): void {
      publish(presenceOutboundSubject(intent), { kind: "outbound", intent });
    },
    async close(): Promise<void> {
      if (nc && !nc.isClosed()) {
        await nc.drain();
      }
    },
  };
}
