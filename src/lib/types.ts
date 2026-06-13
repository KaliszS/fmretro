export type FmEditionInfo = {
  major: number;
  minor: number;
  label: string;
  start_year: number;
};

export type Status = {
  fm_detected: boolean;
  pid: number | null;
  patch_active: boolean;
  shift: number | null;
  min_year: number | null;
  message: string;
  year_table_base: string | null;
  hooks: string[];
  edition: FmEditionInfo | null;
};

export type FmEdition = {
  id: string;
  label: string;
  startYear: number; // first year of the starting season (e.g. 2022 for FM23 → "2022/23")
};

export type ModuleDebug = {
  base: string;
  size: number;
  size_mb: number;
};

export type ProcessDebug = {
  pid: number;
  path: string | null;
  fm_module: ModuleDebug | null;
  vcrt_module: ModuleDebug | null;
};

export type TableDebug = {
  base: string;
  last_entry: string;
  stride: number;
  entry_count: number;
  year_first: number;
  year_last: number;
  season_base: string | null;
};

export type PatchDebug = {
  shift: number;
  saved_entries: number;
};

export type HookDebug = {
  kind: string;
  module: string;
  leaf: string;
  hook_addr: string;
  hook_rva: string | null;
  shellcode_addr: string;
  shellcode_size: number;
  jump_distance: number;
  current_bytes: string | null;
  is_active: boolean;
};

export type DebugInfo = {
  host_os: string;
  host_arch: string;
  patcher_version: string;
  process: ProcessDebug | null;
  table: TableDebug | null;
  patch: PatchDebug | null;
  hooks: HookDebug[];
  last_message: string;
};
