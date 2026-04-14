/**
 * pronunciation.ts — Tech-term pronunciation dictionary for TTS.
 *
 * Replaces acronyms, abbreviations, and technical jargon with
 * phonetic-friendly spellings so TTS engines pronounce them the
 * way engineers actually say them.
 *
 * Usage: import { pronounce } from "./pronunciation";
 *        const ttsText = pronounce(rawText);
 */

// Entries: [pattern, replacement]
// Patterns are case-insensitive word-boundary matches unless noted.
// Replacements use natural speech spellings.
const DICTIONARY: [RegExp, string][] = [
  // ── Formats & languages ──
  [/\bJSON\b/g, "jayson"],
  [/\bjson\b/g, "jayson"],
  [/\bYAML\b/g, "yammel"],
  [/\byaml\b/g, "yammel"],
  [/\bTOML\b/g, "tom-el"],
  [/\btoml\b/g, "tom-el"],
  [/\bSQL\b/g, "sequel"],
  [/\bsql\b/g, "sequel"],
  [/\bNoSQL\b/g, "no-sequel"],
  [/\bGraphQL\b/g, "graph-Q-L"],
  [/\bgRPC\b/g, "G-R-P-C"],
  [/\bWASM\b/gi, "waz-um"],
  [/\bRegex\b/gi, "rej-ex"],

  // ── Tools & platforms ──
  [/\bCLI\b/g, "C-L-I"],
  [/\bAPI\b/g, "A-P-I"],
  [/\bAPIs\b/g, "A-P-eyes"],
  [/\bSDK\b/g, "S-D-K"],
  [/\bSDKs\b/g, "S-D-Ks"],
  [/\bIDE\b/g, "I-D-E"],
  [/\bCI\/CD\b/g, "C-I C-D"],
  [/\bCI\b/g, "C-I"],
  [/\bCD\b/g, "C-D"],
  [/\bK8s\b/gi, "kubernetes"],
  [/\bk8s\b/g, "kubernetes"],
  [/\bHelm\b/g, "helm"],
  [/\bnginx\b/gi, "engine-X"],
  [/\bNGINX\b/g, "engine-X"],
  [/\bDocker\b/g, "docker"],
  [/\bgit\b/g, "git"],
  [/\bGitHub\b/g, "git-hub"],
  [/\bnpm\b/g, "N-P-M"],
  [/\bbun\b/g, "bun"],

  // ── Crypto & Solana ──
  [/\bSOL\b/g, "saul"],
  [/\bSPL\b/g, "S-P-L"],
  [/\bPDA\b/g, "P-D-A"],
  [/\bPDAs\b/g, "P-D-As"],
  [/\bIDL\b/g, "I-D-L"],
  [/\bBPS\b/g, "basis points"],
  [/\bbps\b/g, "basis points"],
  [/\bUSDC\b/g, "U-S-D-C"],
  [/\bUSDT\b/g, "U-S-D-T"],
  [/\bDeFi\b/gi, "dee-fi"],
  [/\bDAO\b/g, "dow"],
  [/\bDAOs\b/g, "dows"],
  [/\bNFT\b/g, "N-F-T"],
  [/\bNFTs\b/g, "N-F-Ts"],
  [/\bTVL\b/g, "T-V-L"],
  [/\bAMM\b/g, "A-M-M"],
  [/\bRPC\b/g, "R-P-C"],
  [/\btxn\b/gi, "transaction"],
  [/\btxns\b/gi, "transactions"],
  [/\bpubkey\b/gi, "pub-key"],
  [/\bpubkeys\b/gi, "pub-keys"],
  [/\bdevnet\b/gi, "dev-net"],
  [/\bmainnet\b/gi, "main-net"],
  [/\btestnet\b/gi, "test-net"],

  // ── Architecture & patterns ──
  [/\bHTTP\b/g, "H-T-T-P"],
  [/\bHTTPS\b/g, "H-T-T-P-S"],
  [/\bREST\b/g, "rest"],
  [/\bRESTful\b/g, "restful"],
  [/\bSSL\b/g, "S-S-L"],
  [/\bTLS\b/g, "T-L-S"],
  [/\bJWT\b/g, "jot"],
  [/\bJWTs\b/g, "jots"],
  [/\bOAuth\b/gi, "oh-auth"],
  [/\bSSO\b/g, "S-S-O"],
  [/\bDNS\b/g, "D-N-S"],
  [/\bCDN\b/g, "C-D-N"],
  [/\bORM\b/g, "O-R-M"],
  [/\bCRUD\b/g, "crud"],
  [/\bSaaS\b/gi, "sass"],
  [/\bIaaS\b/gi, "eye-az"],
  [/\bPaaS\b/gi, "paz"],
  [/\bMVP\b/g, "M-V-P"],
  [/\bPOC\b/g, "P-O-C"],
  [/\bEOD\b/g, "E-O-D"],
  [/\bPR\b/g, "P-R"],
  [/\bPRs\b/g, "P-Rs"],
  [/\bPRD\b/g, "P-R-D"],
  [/\bLLM\b/g, "L-L-M"],
  [/\bLLMs\b/g, "L-L-Ms"],
  [/\bTTS\b/g, "T-T-S"],
  [/\bMCP\b/g, "M-C-P"],
  [/\bCPU\b/g, "C-P-U"],
  [/\bGPU\b/g, "G-P-U"],
  [/\bRAM\b/g, "ram"],
  [/\bSSD\b/g, "S-S-D"],
  [/\bVM\b/g, "V-M"],
  [/\bVMs\b/g, "V-Ms"],
  [/\bIP\b/g, "I-P"],
  [/\bTCP\b/g, "T-C-P"],
  [/\bUDP\b/g, "U-D-P"],
  [/\bWebSocket\b/gi, "web-socket"],
  [/\bWSS\b/g, "W-S-S"],
  [/\bCORS\b/g, "cors"],

  // ── Rust / systems ──
  [/\bRust\b/g, "rust"],
  [/\bcargo\b/gi, "cargo"],
  [/\bclippy\b/gi, "clippy"],
  [/\bstruct\b/g, "struct"],
  [/\bstructs\b/g, "structs"],
  [/\benum\b/g, "ee-num"],
  [/\benums\b/g, "ee-nums"],
  [/\bimpl\b/g, "imple"],
  [/\basync\b/g, "a-sink"],
  [/\bawait\b/g, "a-wait"],
  [/\bmutex\b/gi, "mew-tex"],
  [/\bARC\b/g, "arc"],
  [/\bu8\b/g, "you-eight"],
  [/\bu16\b/g, "you-sixteen"],
  [/\bu32\b/g, "you-thirty-two"],
  [/\bu64\b/g, "you-sixty-four"],
  [/\bi64\b/g, "eye-sixty-four"],
  [/\bf64\b/g, "eff-sixty-four"],

  // ── Common code patterns ──
  [/\bfn\b/g, "function"],
  [/\bpub\b/g, "pub"],
  [/\bconst\b/g, "const"],
  [/\bstdout\b/gi, "standard-out"],
  [/\bstderr\b/gi, "standard-error"],
  [/\bstdin\b/gi, "standard-in"],
  [/\benv\b/g, "environment"],
  [/\bENV\b/g, "environment"],
  [/\bconfig\b/gi, "config"],
  [/\bsudo\b/gi, "sue-doo"],
  [/\bchmod\b/gi, "ch-mod"],
  [/\bchown\b/gi, "ch-own"],
  [/\bmkdir\b/gi, "make-dir"],
  [/\brsync\b/gi, "ar-sync"],

  // ── File extensions (when mentioned as words) ──
  [/\.ts\b/g, " typescript"],
  [/\.js\b/g, " javascript"],
  [/\.rs\b/g, " rust file"],
  [/\.md\b/g, " markdown"],

  // ── Symbols that TTS might read literally ──
  [/`([^`]+)`/g, "$1"],     // strip backticks
  [/\*\*([^*]+)\*\*/g, "$1"], // strip bold
  [/\*([^*]+)\*/g, "$1"],    // strip italics
  [/#{1,6}\s/g, ""],          // strip markdown headers
  [/---+/g, ""],              // strip horizontal rules
  [/```[a-z]*\n?/g, ""],     // strip code fences
];

/**
 * Apply the pronunciation dictionary to text before sending to TTS.
 * Idempotent — safe to call multiple times.
 */
export function pronounce(text: string): string {
  let result = text;
  for (const [pattern, replacement] of DICTIONARY) {
    result = result.replace(pattern, replacement);
  }
  // Collapse multiple spaces
  result = result.replace(/  +/g, " ");
  return result;
}
