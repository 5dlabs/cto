import { createEnvelope, validateAgentEnvelope } from "./index.js";
import type { AgentAddress, AgentEnvelope, AgentGroup, AgentIdentity, AgentMessage, CreateEnvelopeOptions } from "./types.js";

export interface RegisterAgentOptions {
  now?: Date | string | number;
  ttlMs?: number;
  expiresAt?: Date | string | number;
  metadata?: Record<string, string>;
}

export interface RegisteredAgent {
  identity: AgentIdentity;
  registeredAt: string;
  expiresAt: string;
  metadata?: Record<string, string>;
}

export interface RegisterGroupOptions {
  metadata?: Record<string, string>;
}

export interface InboxMessage<TBody = unknown> {
  deliveryId: string;
  recipient: AgentIdentity;
  envelope: AgentEnvelope<TBody>;
  deliveredAt: string;
  attempts: number;
  lastAttemptAt: string;
}

export interface DeadLetterMessage<TBody = unknown> extends InboxMessage<TBody> {
  deadLetteredAt: string;
  reason: string;
}

export interface DeliveryResult<TBody = unknown> {
  envelope: AgentEnvelope<TBody>;
  recipients: AgentIdentity[];
  deliveries: InboxMessage<TBody>[];
}

export interface ReadInboxOptions {
  now?: Date | string | number;
  limit?: number;
  includeExpired?: boolean;
}

export interface SendEnvelopeOptions {
  now?: Date | string | number;
}

export interface SendMessageOptions extends CreateEnvelopeOptions {
  now?: Date | string | number;
}

export interface FailDeliveryOptions {
  now?: Date | string | number;
  reason?: string;
}

export interface InMemoryCoordinationPlaneOptions {
  now?: () => Date | string | number;
  defaultAgentTtlMs?: number;
  maxDeliveryAttempts?: number;
}

const DEFAULT_AGENT_TTL_MS = 5 * 60 * 1000;
const DEFAULT_MAX_DELIVERY_ATTEMPTS = 3;

function toMillis(value: Date | string | number): number {
  const millis = value instanceof Date ? value.getTime() : typeof value === "number" ? value : Date.parse(value);
  if (!Number.isFinite(millis)) {
    throw new Error(`invalid timestamp: ${String(value)}`);
  }
  return millis;
}

function toIso(value: Date | string | number): string {
  return new Date(toMillis(value)).toISOString();
}

function cloneIdentity(identity: AgentIdentity): AgentIdentity {
  return {
    ...identity,
    metadata: identity.metadata === undefined ? undefined : { ...identity.metadata },
  };
}

function cloneRegisteredAgent(agent: RegisteredAgent): RegisteredAgent {
  return {
    identity: cloneIdentity(agent.identity),
    registeredAt: agent.registeredAt,
    expiresAt: agent.expiresAt,
    metadata: agent.metadata === undefined ? undefined : { ...agent.metadata },
  };
}

function cloneEnvelope<TBody>(envelope: AgentEnvelope<TBody>): AgentEnvelope<TBody> {
  return {
    ...envelope,
    message: {
      ...envelope.message,
      from: cloneIdentity(envelope.message.from),
      metadata: envelope.message.metadata === undefined ? undefined : { ...envelope.message.metadata },
    },
    trace: envelope.trace === undefined ? undefined : { ...envelope.trace },
  };
}

function cloneInboxMessage<TBody>(message: InboxMessage<TBody>): InboxMessage<TBody> {
  return {
    ...message,
    recipient: cloneIdentity(message.recipient),
    envelope: cloneEnvelope(message.envelope),
  };
}

function cloneDeadLetterMessage<TBody>(message: DeadLetterMessage<TBody>): DeadLetterMessage<TBody> {
  return {
    ...cloneInboxMessage(message),
    deadLetteredAt: message.deadLetteredAt,
    reason: message.reason,
  };
}

function matchesOptionalScope(identity: AgentIdentity, address: { projectId?: string; taskId?: string }): boolean {
  if (address.projectId !== undefined && identity.projectId !== address.projectId) {
    return false;
  }
  if (address.taskId !== undefined && identity.taskId !== address.taskId) {
    return false;
  }
  return true;
}

function envelopeExpiresAt(envelope: AgentEnvelope): number | undefined {
  if (envelope.ttlMs === undefined) {
    return undefined;
  }
  return toMillis(envelope.sentAt) + envelope.ttlMs;
}

/**
 * Minimal runtime-neutral, in-memory coordination-plane service.
 *
 * The store is intentionally process-local: messages survive until ack, delivery
 * failure/dead-letter, or configured expiry while this object is alive. No
 * Discord/transport payloads or credentials are modeled here.
 */
export class InMemoryCoordinationPlane {
  private readonly agents = new Map<string, RegisteredAgent>();
  private readonly groups = new Map<string, AgentGroup>();
  private readonly inboxes = new Map<string, InboxMessage[]>();
  private readonly deadLetters = new Map<string, DeadLetterMessage[]>();
  private deliverySequence = 0;

  private readonly clock: () => Date | string | number;
  private readonly defaultAgentTtlMs: number;
  private readonly maxDeliveryAttempts: number;

  constructor(options: InMemoryCoordinationPlaneOptions = {}) {
    this.clock = options.now ?? (() => new Date());
    this.defaultAgentTtlMs = options.defaultAgentTtlMs ?? DEFAULT_AGENT_TTL_MS;
    this.maxDeliveryAttempts = options.maxDeliveryAttempts ?? DEFAULT_MAX_DELIVERY_ATTEMPTS;
  }

  registerAgent(identity: AgentIdentity, options: RegisterAgentOptions = {}): RegisteredAgent {
    const now = options.now ?? this.clock();
    const registeredAtMs = toMillis(now);
    const ttlMs = options.ttlMs ?? this.defaultAgentTtlMs;
    const expiresAt = options.expiresAt === undefined ? new Date(registeredAtMs + ttlMs) : options.expiresAt;
    const registered: RegisteredAgent = {
      identity: cloneIdentity(identity),
      registeredAt: new Date(registeredAtMs).toISOString(),
      expiresAt: toIso(expiresAt),
      metadata: options.metadata === undefined ? undefined : { ...options.metadata },
    };

    this.agents.set(identity.agentId, registered);
    if (!this.inboxes.has(identity.agentId)) {
      this.inboxes.set(identity.agentId, []);
    }
    if (!this.deadLetters.has(identity.agentId)) {
      this.deadLetters.set(identity.agentId, []);
    }

    return cloneRegisteredAgent(registered);
  }

  unregisterAgent(agentId: string): boolean {
    return this.agents.delete(agentId);
  }

  registerGroup(group: AgentGroup, options: RegisterGroupOptions = {}): AgentGroup {
    const stored: AgentGroup = {
      ...group,
      members: group.members.map(cloneIdentity),
      metadata: { ...(group.metadata ?? {}), ...(options.metadata ?? {}) },
    };
    this.groups.set(group.groupId, stored);
    return {
      ...stored,
      members: stored.members.map(cloneIdentity),
      metadata: stored.metadata === undefined ? undefined : { ...stored.metadata },
    };
  }

  unregisterGroup(groupId: string): boolean {
    return this.groups.delete(groupId);
  }

  lookup(address: AgentAddress, now: Date | string | number = this.clock()): AgentIdentity[] {
    const nowMs = toMillis(now);
    this.purgeExpiredAgents(nowMs);

    const activeAgents = Array.from(this.agents.values())
      .filter((agent) => toMillis(agent.expiresAt) > nowMs)
      .map((agent) => agent.identity);

    let matches: AgentIdentity[];
    switch (address.kind) {
      case "agent":
        matches = activeAgents.filter(
          (identity) => identity.agentId === address.agentId && matchesOptionalScope(identity, address),
        );
        break;
      case "project":
        matches = activeAgents.filter((identity) => identity.projectId === address.projectId);
        break;
      case "task":
        matches = activeAgents.filter(
          (identity) => identity.projectId === address.projectId && identity.taskId === address.taskId,
        );
        break;
      case "role":
        matches = activeAgents.filter(
          (identity) => identity.role === address.role && matchesOptionalScope(identity, address),
        );
        break;
      case "group": {
        const group = this.groups.get(address.groupId);
        if (group === undefined) {
          matches = [];
          break;
        }
        const memberIds = new Set(group.members.map((member) => member.agentId));
        matches = activeAgents.filter((identity) => memberIds.has(identity.agentId));
        break;
      }
      case "broadcast":
        matches = activeAgents;
        break;
    }

    return this.uniqueByAgentId(matches).map(cloneIdentity);
  }

  sendMessage<TBody>(message: AgentMessage<TBody>, options: SendMessageOptions = {}): DeliveryResult<TBody> {
    const now = options.now ?? this.clock();
    const envelope = createEnvelope(message, {
      envelopeId: options.envelopeId,
      subject: options.subject,
      sentAt: options.sentAt ?? toIso(now),
      replyTo: options.replyTo,
      ttlMs: options.ttlMs,
      attempt: options.attempt,
      trace: options.trace,
    });
    return this.sendEnvelope(envelope, { now });
  }

  sendEnvelope<TBody>(envelope: AgentEnvelope<TBody>, options: SendEnvelopeOptions = {}): DeliveryResult<TBody> {
    const validation = validateAgentEnvelope(envelope);
    if (validation.ok === false) {
      throw new Error(`invalid agent envelope: ${validation.issues.map((entry) => `${entry.path} ${entry.message}`).join("; ")}`);
    }

    const now = options.now ?? this.clock();
    const nowMs = toMillis(now);
    this.purgeExpiredAgents(nowMs);
    this.purgeExpiredMessages(nowMs);

    if (this.isEnvelopeExpired(envelope, nowMs)) {
      return { envelope: cloneEnvelope(envelope), recipients: [], deliveries: [] };
    }

    const recipients = this.lookup(envelope.message.to, nowMs);
    const deliveredAt = new Date(nowMs).toISOString();
    const deliveries = recipients.map((recipient) => {
      const delivery: InboxMessage<TBody> = {
        deliveryId: this.nextDeliveryId(recipient.agentId, envelope.envelopeId),
        recipient: cloneIdentity(recipient),
        envelope: cloneEnvelope(envelope),
        deliveredAt,
        attempts: envelope.attempt ?? 0,
        lastAttemptAt: deliveredAt,
      };
      const inbox = this.inboxes.get(recipient.agentId) ?? [];
      inbox.push(delivery);
      this.inboxes.set(recipient.agentId, inbox);
      return cloneInboxMessage(delivery);
    });

    return {
      envelope: cloneEnvelope(envelope),
      recipients: recipients.map(cloneIdentity),
      deliveries,
    };
  }

  broadcast<TBody>(message: Omit<AgentMessage<TBody>, "to">, options: SendMessageOptions = {}): DeliveryResult<TBody> {
    return this.sendMessage({ ...message, to: { kind: "broadcast" } }, options);
  }

  readInbox(agentId: string, options: ReadInboxOptions = {}): InboxMessage[] {
    const now = options.now ?? this.clock();
    const nowMs = toMillis(now);
    if (!options.includeExpired) {
      this.purgeExpiredMessages(nowMs);
    }

    const messages = (this.inboxes.get(agentId) ?? []).filter(
      (message) => options.includeExpired || !this.isEnvelopeExpired(message.envelope, nowMs),
    );
    const limited = options.limit === undefined ? messages : messages.slice(0, options.limit);
    return limited.map(cloneInboxMessage);
  }

  ack(agentId: string, deliveryIdOrEnvelopeId: string): boolean {
    const inbox = this.inboxes.get(agentId) ?? [];
    const index = inbox.findIndex(
      (message) => message.deliveryId === deliveryIdOrEnvelopeId || message.envelope.envelopeId === deliveryIdOrEnvelopeId,
    );
    if (index === -1) {
      return false;
    }
    inbox.splice(index, 1);
    this.inboxes.set(agentId, inbox);
    return true;
  }

  failDelivery(agentId: string, deliveryIdOrEnvelopeId: string, options: FailDeliveryOptions = {}): InboxMessage | DeadLetterMessage | undefined {
    const inbox = this.inboxes.get(agentId) ?? [];
    const index = inbox.findIndex(
      (message) => message.deliveryId === deliveryIdOrEnvelopeId || message.envelope.envelopeId === deliveryIdOrEnvelopeId,
    );
    if (index === -1) {
      return undefined;
    }

    const now = options.now ?? this.clock();
    const nowIso = toIso(now);
    const message = inbox[index];
    const attempts = message.attempts + 1;
    const updated: InboxMessage = {
      ...message,
      attempts,
      lastAttemptAt: nowIso,
      envelope: {
        ...message.envelope,
        attempt: attempts,
      },
    };

    if (attempts >= this.maxDeliveryAttempts) {
      const deadLetter: DeadLetterMessage = {
        ...updated,
        deadLetteredAt: nowIso,
        reason: options.reason ?? "max delivery attempts exceeded",
      };
      inbox.splice(index, 1);
      this.inboxes.set(agentId, inbox);
      const deadLetters = this.deadLetters.get(agentId) ?? [];
      deadLetters.push(deadLetter);
      this.deadLetters.set(agentId, deadLetters);
      return cloneDeadLetterMessage(deadLetter);
    }

    inbox[index] = updated;
    this.inboxes.set(agentId, inbox);
    return cloneInboxMessage(updated);
  }

  readDeadLetters(agentId: string): DeadLetterMessage[] {
    return (this.deadLetters.get(agentId) ?? []).map(cloneDeadLetterMessage);
  }

  purgeExpired(now: Date | string | number = this.clock()): void {
    const nowMs = toMillis(now);
    this.purgeExpiredAgents(nowMs);
    this.purgeExpiredMessages(nowMs);
  }

  private purgeExpiredAgents(nowMs: number): void {
    for (const [agentId, agent] of Array.from(this.agents.entries())) {
      if (toMillis(agent.expiresAt) <= nowMs) {
        this.agents.delete(agentId);
      }
    }
  }

  private purgeExpiredMessages(nowMs: number): void {
    for (const [agentId, inbox] of Array.from(this.inboxes.entries())) {
      this.inboxes.set(
        agentId,
        inbox.filter((message) => !this.isEnvelopeExpired(message.envelope, nowMs)),
      );
    }
  }

  private isEnvelopeExpired(envelope: AgentEnvelope, nowMs: number): boolean {
    const expiresAt = envelopeExpiresAt(envelope);
    return expiresAt !== undefined && expiresAt <= nowMs;
  }

  private uniqueByAgentId(agents: AgentIdentity[]): AgentIdentity[] {
    const seen = new Set<string>();
    const unique: AgentIdentity[] = [];
    for (const agent of agents) {
      if (!seen.has(agent.agentId)) {
        seen.add(agent.agentId);
        unique.push(agent);
      }
    }
    return unique;
  }

  private nextDeliveryId(agentId: string, envelopeId: string): string {
    this.deliverySequence += 1;
    return `${agentId}:${envelopeId}:${this.deliverySequence}`;
  }
}
