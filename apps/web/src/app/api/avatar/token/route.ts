import { NextRequest, NextResponse } from 'next/server';
import {
  AccessToken,
  AgentDispatchClient,
  RoomServiceClient,
  TwirpError,
} from 'livekit-server-sdk';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

const DEFAULT_AGENT_NAME = 'morgan-avatar';
const ROOM_NAME_PATTERN = /^[A-Za-z0-9_-]{1,64}$/;

function isAlreadyExistsError(error: unknown): boolean {
  return error instanceof TwirpError && error.code === 'already_exists';
}

function getAvatarAgentName(): string {
  return (
    process.env.MORGAN_AVATAR_AGENT_NAME?.trim() ||
    process.env.MORGAN_AGENT_NAME?.trim() ||
    DEFAULT_AGENT_NAME
  );
}

function getRoomName(roomName?: string): string | null {
  if (!roomName?.trim()) {
    return `morgan-${crypto.randomUUID().slice(0, 8)}`;
  }

  const normalized = roomName.trim();
  return ROOM_NAME_PATTERN.test(normalized) ? normalized : null;
}

async function ensureRoomAndDispatch(params: {
  apiKey: string;
  apiSecret: string;
  agentName: string;
  roomName: string;
  wsUrl: string;
}) {
  const { apiKey, apiSecret, agentName, roomName, wsUrl } = params;
  const roomClient = new RoomServiceClient(wsUrl, apiKey, apiSecret);

  try {
    await roomClient.createRoom({
      name: roomName,
      emptyTimeout: 60,
      departureTimeout: 15,
      maxParticipants: 4,
    });
  } catch (error) {
    if (!isAlreadyExistsError(error)) {
      throw error;
    }
  }

  const dispatchClient = new AgentDispatchClient(wsUrl, apiKey, apiSecret);
  const dispatches = await dispatchClient.listDispatch(roomName);
  const hasMorganDispatch = dispatches.some((dispatch) => dispatch.agentName === agentName);

  if (!hasMorganDispatch) {
    await dispatchClient.createDispatch(roomName, agentName);
  }
}

export async function POST(request: NextRequest) {
  const apiKey = process.env.LIVEKIT_API_KEY;
  const apiSecret = process.env.LIVEKIT_API_SECRET;
  const wsUrl = process.env.LIVEKIT_URL;

  if (!apiKey || !apiSecret || !wsUrl) {
    return NextResponse.json(
      { error: 'Server misconfigured. Set LIVEKIT_URL, LIVEKIT_API_KEY, and LIVEKIT_API_SECRET.' },
      { status: 500 }
    );
  }

  const body = (await request.json().catch(() => ({}))) as {
    participantName?: string;
    roomName?: string;
  };
  const roomName = getRoomName(body.roomName);

  if (!roomName) {
    return NextResponse.json(
      { error: 'Room names may only contain letters, numbers, underscores, and dashes.' },
      { status: 400 }
    );
  }

  const identity = `user-${crypto.randomUUID().slice(0, 8)}`;
  const agentName = getAvatarAgentName();

  try {
    await ensureRoomAndDispatch({
      apiKey,
      apiSecret,
      agentName,
      roomName,
      wsUrl,
    });
  } catch (error) {
    const message = error instanceof Error ? error.message : 'Unable to prepare the LiveKit room.';
    return NextResponse.json({ error: message }, { status: 500 });
  }

  const token = new AccessToken(apiKey, apiSecret, { identity, ttl: '10m' });
  token.addGrant({
    roomJoin: true,
    room: roomName,
    canPublish: true,
    canSubscribe: true,
  });

  return NextResponse.json(
    {
      token: await token.toJwt(),
      serverUrl: wsUrl,
      roomName,
      identity,
      participantName: typeof body.participantName === 'string' ? body.participantName : undefined,
    },
    {
      headers: {
        'Cache-Control': 'no-store',
      },
    }
  );
}
