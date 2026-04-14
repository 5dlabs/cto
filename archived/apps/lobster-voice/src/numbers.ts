const ONES = [
  "", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
  "ten", "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen",
  "seventeen", "eighteen", "nineteen",
];

const TENS = [
  "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

const SCALES = ["", "thousand", "million", "billion", "trillion"];

function threeDigits(n: number): string {
  if (n === 0) return "";
  if (n < 20) return ONES[n];
  if (n < 100) {
    const t = TENS[Math.floor(n / 10)];
    const o = ONES[n % 10];
    return o ? `${t} ${o}` : t;
  }
  const h = `${ONES[Math.floor(n / 100)]} hundred`;
  const rem = n % 100;
  return rem ? `${h} and ${threeDigits(rem)}` : h;
}

function integerToWords(n: number): string {
  if (n === 0) return "zero";
  if (n < 0) return `negative ${integerToWords(-n)}`;

  const groups: string[] = [];
  let remaining = Math.abs(n);
  let scaleIdx = 0;

  while (remaining > 0) {
    const chunk = remaining % 1000;
    if (chunk !== 0) {
      const words = threeDigits(chunk);
      const scale = SCALES[scaleIdx];
      groups.unshift(scale ? `${words} ${scale}` : words);
    }
    remaining = Math.floor(remaining / 1000);
    scaleIdx++;
  }

  return groups.join(", ");
}

const UUID_RE = /\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/gi;
const LONG_NUM_RE = /\b(\d{5,})\b/g;
const TIMESTAMP_RE = /\b(1[6-9]\d{8})\b/g;

function abbreviateUuid(uuid: string): string {
  const short = uuid.slice(0, 8);
  return `ID ${short}`;
}

function humanizeTimestamp(ts: string): string {
  const num = parseInt(ts, 10);
  const date = new Date(num * 1000);
  if (isNaN(date.getTime())) return ts;

  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.round(diffMs / 60_000);

  if (diffMin >= 0 && diffMin < 2) return "just now";
  if (diffMin >= 2 && diffMin < 60) return `${diffMin} minutes ago`;
  if (diffMin >= 60 && diffMin < 1440) {
    const hours = Math.round(diffMin / 60);
    return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  }

  return date.toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function humanizeNumbers(text: string): string {
  let result = text;

  result = result.replace(UUID_RE, (m) => abbreviateUuid(m));

  result = result.replace(TIMESTAMP_RE, (m) => {
    const num = parseInt(m, 10);
    if (num >= 1_600_000_000 && num <= 2_000_000_000) {
      return humanizeTimestamp(m);
    }
    return m;
  });

  result = result.replace(LONG_NUM_RE, (m) => {
    const num = parseInt(m, 10);
    if (num > 99_999 && num <= Number.MAX_SAFE_INTEGER) {
      return integerToWords(num);
    }
    return m;
  });

  return result;
}
