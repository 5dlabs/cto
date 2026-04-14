import { spawnSync } from "child_process";
import { existsSync } from "fs";
import { dirname, join } from "path";

export function playAudio(filePath: string): boolean {
  if (!existsSync(filePath)) return false;

  if (process.platform === "darwin") {
    const deviceId = process.env.LOBSTER_VOICE_AUDIO_DEVICE;
    if (deviceId) {
      const ptd = join(dirname(dirname(import.meta.path)), "play-to-device");
      if (existsSync(ptd)) {
        const r = spawnSync(ptd, [filePath, deviceId], { stdio: "inherit", timeout: 30_000 });
        if (r.status === 0) return true;
      }
    }
    const r = spawnSync("afplay", [filePath], { stdio: "inherit", timeout: 30_000 });
    return r.status === 0;
  }

  for (const player of ["ffplay", "aplay", "paplay"]) {
    try {
      const args = player === "ffplay"
        ? ["-nodisp", "-autoexit", "-loglevel", "quiet", filePath]
        : [filePath];
      const r = spawnSync(player, args, { stdio: "inherit", timeout: 30_000 });
      if (r.status === 0) return true;
    } catch {
      continue;
    }
  }

  return false;
}
