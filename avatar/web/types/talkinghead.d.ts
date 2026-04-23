declare module "@met4citizen/talkinghead" {
  /**
   * Minimal ambient types for the TalkingHead class.
   *
   * Covers only the methods we currently call. The upstream package ships
   * no type declarations; expand this file as more of the API is used.
   * Reference:
   *   https://github.com/met4citizen/TalkingHead/blob/main/README.md
   */

  export type TalkingHeadOptions = {
    ttsEndpoint?: string | null;
    ttsApikey?: string | null;
    ttsLang?: string;
    ttsVoice?: string;
    jwsGet?: (() => Promise<string>) | null;
    lipsyncModules?: string[];
    lipsyncLang?: string;
    modelFPS?: number;
    modelPixelRatio?: number;
    modelMovementFactor?: number;
    modelRoot?: string;
    dracoEnabled?: boolean;
    dracoDecoderPath?: string;
    cameraView?: "full" | "mid" | "upper" | "head";
    cameraDistance?: number;
    cameraX?: number;
    cameraY?: number;
    cameraRotateX?: number;
    cameraRotateY?: number;
    cameraRotateEnable?: boolean;
    cameraPanEnable?: boolean;
    cameraZoomEnable?: boolean;
    lightAmbientColor?: number | string;
    lightAmbientIntensity?: number;
    lightDirectColor?: number | string;
    lightDirectIntensity?: number;
    lightDirectPhi?: number;
    lightDirectTheta?: number;
    lightSpotColor?: number | string;
    lightSpotIntensity?: number;
    lightSpotPhi?: number;
    lightSpotTheta?: number;
    lightSpotDispersion?: number;
    avatarMood?: "neutral" | "happy" | "angry" | "sad" | "fear" | "disgust" | "love" | "sleep";
    avatarMute?: boolean;
    avatarIdleEyeContact?: number;
    avatarIdleHeadMove?: number;
    avatarSpeakingEyeContact?: number;
    avatarSpeakingHeadMove?: number;
    avatarIgnoreCamera?: boolean;
    mixerGainSpeech?: number | null;
    mixerGainBackground?: number | null;
    pcmSampleRate?: number;
    audioCtx?: AudioContext | null;
    update?: ((delta: number) => void) | null;
  };

  export type TalkingHeadAvatarOptions = {
    url: string;
    body?: "M" | "F";
    avatarMood?: TalkingHeadOptions["avatarMood"];
    baseline?: Record<string, number>;
    retarget?: Record<string, unknown>;
    modelDynamicBones?: unknown[];
    lipsyncLang?: string;
    ttsLang?: string;
    ttsVoice?: string;
    avatarIdleEyeContact?: number;
    avatarSpeakingEyeContact?: number;
    avatarListeningEyeContact?: number;
    avatarSpeakingHeadMove?: number;
    avatarIgnoreCamera?: boolean;
  };

  export type OculusViseme =
    | "sil"
    | "PP"
    | "FF"
    | "TH"
    | "DD"
    | "kk"
    | "CH"
    | "SS"
    | "nn"
    | "RR"
    | "aa"
    | "E"
    | "I"
    | "O"
    | "U";

  export type TalkingHeadSpeakAudioPayload = {
    audio: AudioBuffer | ArrayBuffer[] | Int16Array[];
    words?: string[];
    wtimes?: number[];
    wdurations?: number[];
    visemes?: OculusViseme[];
    vtimes?: number[];
    vdurations?: number[];
    markers?: Array<() => void>;
    mtimes?: number[];
    anim?: unknown;
  };

  export type TalkingHeadProgressFn = (
    url: string,
    event: { lengthComputable: boolean; loaded: number; total: number },
  ) => void;

  export class TalkingHead {
    constructor(node: HTMLElement, options?: TalkingHeadOptions);

    showAvatar(
      avatar: TalkingHeadAvatarOptions,
      onprogress?: TalkingHeadProgressFn,
    ): Promise<void>;

    setView(
      view: "full" | "mid" | "upper" | "head",
      opt?: Record<string, number>,
    ): void;

    setLighting(opt: Record<string, number | string>): void;

    speakText(
      text: string,
      opt?: Record<string, unknown>,
      onsubtitles?: ((subtitle: string) => void) | null,
      excludes?: number[][],
    ): void;

    speakAudio(
      audio: TalkingHeadSpeakAudioPayload,
      opt?: Record<string, unknown>,
      onsubtitles?: ((subtitle: string) => void) | null,
    ): void;

    setMood(mood: NonNullable<TalkingHeadOptions["avatarMood"]>): void;

    stopSpeaking(): void;

    playGesture(name: string, dur?: number, mirror?: boolean, ms?: number): void;
    stopGesture(ms?: number): void;

    lookAtCamera(t: number): void;
    lookAhead(t: number): void;
    makeEyeContact(t: number): void;

    start(): void;
    stop(): void;
  }
}
