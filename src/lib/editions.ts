import type { FmEdition } from "./types";

// Known FM editions for nicer labels. Any other major is supported on the fly.
export const EDITIONS: FmEdition[] = [
  { id: "fm20", label: "FM 2020", startYear: 2019 },
  { id: "fm21", label: "FM 2021", startYear: 2020 },
  { id: "fm22", label: "FM 2022", startYear: 2021 },
  { id: "fm23", label: "FM 2023", startYear: 2022 },
  { id: "fm24", label: "FM 2024", startYear: 2023 },
  { id: "fm25", label: "FM 2025", startYear: 2024 },
  { id: "fm26", label: "FM 2026", startYear: 2025 },
];

function buildEdition(major: number): FmEdition {
  return {
    id: `fm${major.toString().padStart(2, "0")}`,
    label: `FM 20${major.toString().padStart(2, "0")}`,
    startYear: 2000 + major - 1,
  };
}

/** Resolve an edition by id; constructs an entry on the fly for unknown majors. */
export function findEdition(id: string): FmEdition {
  const known = EDITIONS.find((e) => e.id === id);
  if (known) return known;
  const m = id.match(/^fm(\d{1,2})$/i);
  if (m) return buildEdition(parseInt(m[1], 10));
  return EDITIONS[EDITIONS.length - 1];
}

/** Map any FM major (1-99) → chip id, e.g. 24 → "fm24", 31 → "fm31". */
export function editionIdForMajor(major: number): string | null {
  if (!Number.isInteger(major) || major < 1 || major >= 100) return null;
  return `fm${major.toString().padStart(2, "0")}`;
}

export function formatSeason(year: number): string {
  const next = (year + 1) % 100;
  return `${year}/${next.toString().padStart(2, "0")}`;
}
