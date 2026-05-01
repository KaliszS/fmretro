import { invoke } from "@tauri-apps/api/core";
import type { DebugInfo, Status } from "./types";

export async function detectFm(): Promise<number | null> {
  return invoke<number | null>("detect_fm");
}

export async function getStatus(): Promise<Status> {
  return invoke<Status>("get_status");
}

export async function getDebug(): Promise<DebugInfo> {
  return invoke<DebugInfo>("get_debug");
}

export async function applyPatch(
  shiftBack: number,
  minYear: number,
): Promise<string> {
  return invoke<string>("apply_patch", { shiftBack, minYear });
}

export async function restorePatch(): Promise<string> {
  return invoke<string>("restore_patch");
}
