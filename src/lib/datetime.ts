const FRACTIONAL_SECONDS_RE = /^(.+T\d{2}:\d{2}:\d{2})\.(\d{3})\d+([zZ]|[+-]\d{2}:\d{2})$/;

export function normalizeDateInput(value: string | null | undefined): string {
  if (!value) return "";
  return value.replace(FRACTIONAL_SECONDS_RE, "$1.$2$3");
}

export function parseDate(value: string | null | undefined): Date {
  return new Date(normalizeDateInput(value));
}

export function parseDateTimestamp(value: string | null | undefined): number {
  return parseDate(value).getTime();
}

export function elapsedSeconds(
  startedAt: string | null | undefined,
  endedAt: string | null | undefined,
): number {
  const startMs = parseDateTimestamp(startedAt);
  const endMs = parseDateTimestamp(endedAt);

  if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
    return 0;
  }

  return (endMs - startMs) / 1000;
}
