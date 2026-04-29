import { mkdir, writeFile, rm, access, readFile } from 'node:fs/promises';
import { dirname, join, resolve, isAbsolute, basename } from 'node:path';

// Deliberation video generator.
//
// Pipeline:
//   1. Per-chunk ElevenLabs TTS (parallel) -> mp3 files in a temp dir.
//   2. Per-chunk ffmpeg: portrait still + mp3 -> short mp4 segment.
//   3. ffmpeg concat demuxer -> final deliberation.mp4.
//
// No talking-head lipsync — just portrait slideshow timed to narration.

export type DeliberationCharacter =
  | 'morgan'
  | 'optimus'
  | 'pessimus'
  | 'praxis'
  | 'rook'
  | 'veritas';

const CHARACTERS: readonly DeliberationCharacter[] = [
  'morgan',
  'optimus',
  'pessimus',
  'praxis',
  'rook',
  'veritas',
] as const;

// Morgan's voice is locked. Other voices come from intake/voices.json.
const MORGAN_LOCKED_VOICE_ID = 'iP95p4xoKVk53GoZ742B';

const ELEVENLABS_MODEL_ID = 'eleven_turbo_v2_5';
const ELEVENLABS_BASE_URL = 'https://api.elevenlabs.io/v1/text-to-speech';

// Output video parameters. Portraits are stills, so a low-ish framerate is fine.
const VIDEO_WIDTH = 1080;
const VIDEO_HEIGHT = 1920; // portrait 9:16
const VIDEO_FPS = 30;

export interface ChunkInput {
  character: DeliberationCharacter;
  text: string;
}

export interface GenerateDeliberationVideoInput {
  transcript: ChunkInput[];
  outputPath: string;
}

export interface GenerateDeliberationVideoResult {
  outputPath: string;
  segmentCount: number;
  totalDurationSeconds: number;
}

interface VoiceMap {
  morgan: string;
  optimus: string;
  pessimus: string;
  praxis: string;
  rook: string;
  veritas: string;
}

function isCharacter(value: string): value is DeliberationCharacter {
  return (CHARACTERS as readonly string[]).includes(value);
}

async function fileExists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function findRepoRoot(start: string): Promise<string> {
  let cur = resolve(start);
  while (true) {
    if (await fileExists(join(cur, '.git'))) return cur;
    const parent = dirname(cur);
    if (parent === cur) return resolve(start); // fallback
    cur = parent;
  }
}

async function loadVoiceIds(): Promise<VoiceMap> {
  const overridePath = process.env['INTAKE_VOICES_FILE'];
  const repoRoot = await findRepoRoot(process.cwd());
  const voicesPath = overridePath
    ? (isAbsolute(overridePath) ? overridePath : resolve(repoRoot, overridePath))
    : resolve(repoRoot, 'intake/voices.json');

  if (!(await fileExists(voicesPath))) {
    throw new Error(
      `Missing voice ID file at ${voicesPath}. Create intake/voices.json with keys: ` +
        `morgan, optimus, pessimus, praxis, rook, veritas (string voice IDs from ElevenLabs).`
    );
  }

  let raw: string;
  try {
    raw = await readFile(voicesPath, 'utf8');
  } catch (err) {
    throw new Error(`Failed to read ${voicesPath}: ${(err as Error).message}`);
  }

  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(raw) as Record<string, unknown>;
  } catch (err) {
    throw new Error(`Invalid JSON in ${voicesPath}: ${(err as Error).message}`);
  }

  const result: Partial<VoiceMap> = {};
  for (const c of CHARACTERS) {
    const v = parsed[c];
    if (typeof v !== 'string' || v.trim().length === 0) {
      // Morgan has a locked default if missing.
      if (c === 'morgan') {
        result.morgan = MORGAN_LOCKED_VOICE_ID;
        continue;
      }
      throw new Error(
        `Voice ID for "${c}" missing or not a string in ${voicesPath}. ` +
          `Add { "${c}": "<elevenlabs-voice-id>" }.`
      );
    }
    result[c] = v;
  }
  // Morgan is always pinned to the locked voice ID regardless of what the file says.
  result.morgan = MORGAN_LOCKED_VOICE_ID;
  return result as VoiceMap;
}

async function resolvePortraitPaths(): Promise<Record<DeliberationCharacter, string>> {
  const dir = process.env['INTAKE_PORTRAIT_DIR'];
  if (!dir || dir.trim().length === 0) {
    throw new Error(
      'INTAKE_PORTRAIT_DIR env var is not set. Point it at a directory containing ' +
        'morgan.png, optimus.png, pessimus.png, praxis.png, rook.png, veritas.png.'
    );
  }

  const baseDir = isAbsolute(dir) ? dir : resolve(process.cwd(), dir);
  const map = {} as Record<DeliberationCharacter, string>;
  const missing: string[] = [];

  for (const c of CHARACTERS) {
    let found: string | null = null;
    for (const ext of ['png', 'jpg', 'jpeg']) {
      const candidate = join(baseDir, `${c}.${ext}`);
      if (await fileExists(candidate)) {
        found = candidate;
        break;
      }
    }
    if (!found) {
      missing.push(c);
    } else {
      map[c] = found;
    }
  }

  if (missing.length > 0) {
    throw new Error(
      `Portrait files missing in ${baseDir} for: ${missing.join(', ')}. ` +
        `Expected <character>.{png|jpg|jpeg}.`
    );
  }

  return map;
}

async function ttsSynthesize(args: {
  apiKey: string;
  voiceId: string;
  text: string;
  outputPath: string;
}): Promise<void> {
  const url = `${ELEVENLABS_BASE_URL}/${args.voiceId}`;
  const res = await fetch(url, {
    method: 'POST',
    headers: {
      'xi-api-key': args.apiKey,
      'Content-Type': 'application/json',
      Accept: 'audio/mpeg',
    },
    body: JSON.stringify({
      text: args.text,
      model_id: ELEVENLABS_MODEL_ID,
      voice_settings: {
        stability: 0.5,
        similarity_boost: 0.75,
        style: 0.0,
        use_speaker_boost: true,
      },
    }),
  });

  if (!res.ok) {
    const body = await res.text().catch(() => '');
    throw new Error(
      `ElevenLabs TTS failed for voice ${args.voiceId}: HTTP ${res.status} ${res.statusText}` +
        (body ? ` — ${body.slice(0, 500)}` : '')
    );
  }

  const buf = new Uint8Array(await res.arrayBuffer());
  if (buf.byteLength === 0) {
    throw new Error(`ElevenLabs TTS returned empty audio for voice ${args.voiceId}`);
  }
  await writeFile(args.outputPath, buf);
}

async function runProcess(cmd: string[], opts: { captureStdout?: boolean } = {}): Promise<string> {
  const proc = Bun.spawn(cmd, {
    stdout: opts.captureStdout ? 'pipe' : 'inherit',
    stderr: 'pipe',
  });
  const stderrPromise = new Response(proc.stderr).text();
  const stdoutPromise = opts.captureStdout
    ? new Response(proc.stdout).text()
    : Promise.resolve('');
  const exitCode = await proc.exited;
  const [stdout, stderr] = await Promise.all([stdoutPromise, stderrPromise]);
  if (exitCode !== 0) {
    throw new Error(
      `Process failed (${cmd[0]} exit ${exitCode}): ${cmd.join(' ')}\nstderr: ${stderr.slice(0, 1000)}`
    );
  }
  return stdout;
}

async function probeAudioDuration(audioPath: string): Promise<number> {
  const out = await runProcess(
    [
      'ffprobe',
      '-v',
      'error',
      '-show_entries',
      'format=duration',
      '-of',
      'default=noprint_wrappers=1:nokey=1',
      audioPath,
    ],
    { captureStdout: true }
  );
  const trimmed = out.trim();
  const dur = Number(trimmed);
  if (!Number.isFinite(dur) || dur <= 0) {
    throw new Error(`ffprobe returned invalid duration "${trimmed}" for ${audioPath}`);
  }
  return dur;
}

async function buildSegment(args: {
  portraitPath: string;
  audioPath: string;
  durationSeconds: number;
  outputPath: string;
}): Promise<void> {
  // Loop the still image for the audio's duration; scale+pad to canonical portrait
  // dimensions so concat doesn't fail on differing resolutions.
  const vf =
    `scale=${VIDEO_WIDTH}:${VIDEO_HEIGHT}:force_original_aspect_ratio=decrease,` +
    `pad=${VIDEO_WIDTH}:${VIDEO_HEIGHT}:(ow-iw)/2:(oh-ih)/2:color=black,` +
    `setsar=1,format=yuv420p`;

  await runProcess([
    'ffmpeg',
    '-y',
    '-loop',
    '1',
    '-framerate',
    String(VIDEO_FPS),
    '-i',
    args.portraitPath,
    '-i',
    args.audioPath,
    '-t',
    args.durationSeconds.toFixed(3),
    '-vf',
    vf,
    '-r',
    String(VIDEO_FPS),
    '-c:v',
    'libx264',
    '-preset',
    'medium',
    '-tune',
    'stillimage',
    '-pix_fmt',
    'yuv420p',
    '-c:a',
    'aac',
    '-b:a',
    '192k',
    '-ar',
    '48000',
    '-ac',
    '2',
    '-shortest',
    '-movflags',
    '+faststart',
    args.outputPath,
  ]);
}

async function concatSegments(args: {
  segmentPaths: string[];
  listFilePath: string;
  outputPath: string;
}): Promise<void> {
  const lines = args.segmentPaths
    .map((p) => `file '${p.replace(/'/g, "'\\''")}'`)
    .join('\n');
  await writeFile(args.listFilePath, lines + '\n', 'utf8');

  await runProcess([
    'ffmpeg',
    '-y',
    '-f',
    'concat',
    '-safe',
    '0',
    '-i',
    args.listFilePath,
    '-c',
    'copy',
    '-movflags',
    '+faststart',
    args.outputPath,
  ]);
}

export async function generateDeliberationVideo(
  input: GenerateDeliberationVideoInput
): Promise<GenerateDeliberationVideoResult> {
  // -------- validation --------
  if (!input || !Array.isArray(input.transcript) || input.transcript.length === 0) {
    throw new Error('generateDeliberationVideo: transcript must be a non-empty array');
  }
  if (!input.outputPath || typeof input.outputPath !== 'string') {
    throw new Error('generateDeliberationVideo: outputPath is required');
  }

  for (let i = 0; i < input.transcript.length; i++) {
    const chunk = input.transcript[i];
    if (!chunk || typeof chunk !== 'object') {
      throw new Error(`Transcript chunk at index ${i} is not an object`);
    }
    if (typeof chunk.character !== 'string' || !isCharacter(chunk.character)) {
      throw new Error(
        `Transcript chunk at index ${i} has unknown character "${String(chunk.character)}". ` +
          `Allowed: ${CHARACTERS.join(', ')}.`
      );
    }
    if (typeof chunk.text !== 'string' || chunk.text.trim().length === 0) {
      throw new Error(`Transcript chunk at index ${i} has empty text`);
    }
  }

  const apiKey = process.env['ELEVENLABS_API_KEY'];
  if (!apiKey || apiKey.trim().length === 0) {
    throw new Error('ELEVENLABS_API_KEY env var is required for TTS narration');
  }

  const [voices, portraits] = await Promise.all([loadVoiceIds(), resolvePortraitPaths()]);

  // -------- workspace --------
  const outputPath = isAbsolute(input.outputPath)
    ? input.outputPath
    : resolve(process.cwd(), input.outputPath);
  const outputDir = dirname(outputPath);
  await mkdir(outputDir, { recursive: true });

  // Scratch dir lives next to the output (project-relative), never under /tmp.
  const baseName = basename(outputPath, '.mp4') || 'deliberation';
  const workDir = join(outputDir, `.${baseName}-scratch-${Date.now()}`);
  await mkdir(workDir, { recursive: true });

  try {
    // -------- TTS in parallel --------
    const audioPaths: string[] = input.transcript.map((_, i) =>
      join(workDir, `chunk-${String(i).padStart(4, '0')}.mp3`)
    );

    await Promise.all(
      input.transcript.map((chunk, i) => {
        const voiceId = voices[chunk.character];
        const audioPath = audioPaths[i];
        if (!audioPath) {
          // Should never happen — array length matches.
          throw new Error(`Internal error: missing audio path slot for index ${i}`);
        }
        return ttsSynthesize({
          apiKey,
          voiceId,
          text: chunk.text,
          outputPath: audioPath,
        });
      })
    );

    // -------- per-segment ffmpeg (sequential to avoid CPU thrash) --------
    const segmentPaths: string[] = [];
    let totalDuration = 0;
    for (let i = 0; i < input.transcript.length; i++) {
      const chunk = input.transcript[i]!;
      const audioPath = audioPaths[i]!;
      const segmentPath = join(workDir, `segment-${String(i).padStart(4, '0')}.mp4`);
      const duration = await probeAudioDuration(audioPath);
      totalDuration += duration;
      await buildSegment({
        portraitPath: portraits[chunk.character],
        audioPath,
        durationSeconds: duration,
        outputPath: segmentPath,
      });
      segmentPaths.push(segmentPath);
    }

    // -------- concat --------
    const listFile = join(workDir, 'concat.txt');
    await concatSegments({
      segmentPaths,
      listFilePath: listFile,
      outputPath,
    });

    return {
      outputPath,
      segmentCount: segmentPaths.length,
      totalDurationSeconds: Number(totalDuration.toFixed(3)),
    };
  } finally {
    if (process.env['INTAKE_DELIBERATION_KEEP_WORKDIR'] !== '1') {
      await rm(workDir, { recursive: true, force: true }).catch(() => {});
    }
  }
}
