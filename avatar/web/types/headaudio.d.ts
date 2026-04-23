declare module "@met4citizen/headaudio/dist/headaudio.min.mjs" {
  // HeadAudio Oculus viseme names (must match TalkingHead's mtAvatar keys).
  export type HeadAudioVisemeKey =
    | "viseme_aa"
    | "viseme_E"
    | "viseme_I"
    | "viseme_O"
    | "viseme_U"
    | "viseme_PP"
    | "viseme_SS"
    | "viseme_TH"
    | "viseme_DD"
    | "viseme_FF"
    | "viseme_kk"
    | "viseme_nn"
    | "viseme_RR"
    | "viseme_CH"
    | "viseme_sil";

  export type HeadAudioProcessorOptions = {
    frameEventsEnabled?: boolean;
    vadEventsEnabled?: boolean;
    featureEventsEnabled?: boolean;
    visemeEventsEnabled?: boolean;
  };

  export type HeadAudioParameterData = {
    vadMode?: number;
    vadGateActiveDb?: number;
    vadGateActiveMs?: number;
    vadGateInactiveDb?: number;
    vadGateInactiveMs?: number;
    silMode?: number;
    silCalibrationWindowSec?: number;
    silSensitivity?: number;
    speakerMeanHz?: number;
  };

  export type HeadAudioNodeOptions = {
    processorOptions?: HeadAudioProcessorOptions;
    parameterData?: HeadAudioParameterData;
  };

  /**
   * AudioWorkletNode subclass that runs the HeadAudio viseme classifier.
   * Exposed members mirror the README; only what we call is typed.
   */
  export class HeadAudio extends AudioWorkletNode {
    constructor(audioCtx: AudioContext, options?: HeadAudioNodeOptions);

    loadModel(url: string): Promise<void>;

    update(deltaMs: number): void;

    onvalue: ((key: HeadAudioVisemeKey, value: number) => void) | null;
    onstarted: ((data: { event: "start"; t: number }) => void) | null;
    onended: ((data: { event: "end"; t: number }) => void) | null;
    onprocessorerror: ((event: Event) => void) | null;
  }
}
