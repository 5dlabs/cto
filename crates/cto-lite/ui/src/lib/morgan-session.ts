export type MorganModality = 'chat' | 'call' | 'video'

export interface MorganSessionState {
  projectId: string
  agentId: string
  sessionId: string
  roomName: string
  gatewaySessionKey: string | null
  latestUserText: string
  latestAgentText: string
  connectionState: 'idle' | 'connecting' | 'connected' | 'error'
  voiceState: string
  latestTransport: MorganModality | null
  revision: number
}

export function createMorganSessionId(agentId: string, projectId: string): string {
  return `${agentId}-${projectId}`
}

export function createMorganSessionState(
  projectId: string,
  agentId: string
): MorganSessionState {
  const sessionId = createMorganSessionId(agentId, projectId)

  return {
    projectId,
    agentId,
    sessionId,
    roomName: sessionId,
    gatewaySessionKey: null,
    latestUserText: '',
    latestAgentText: '',
    connectionState: 'idle',
    voiceState: 'idle',
    latestTransport: null,
    revision: 0,
  }
}
