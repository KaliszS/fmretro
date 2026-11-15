// Motion helpers. Svelte transitions run through the Web Animations API,
// so the CSS reduced-motion override does not reach them — scale durations here.
import { cubicOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

export function prefersReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Returns `ms`, or 0 when the user asked for reduced motion. */
export function dur(ms: number): number {
  return prefersReducedMotion() ? 0 : ms;
}

/** Fade + short vertical slide, used for swapping views and panels. */
export function rise(
  _node: Element,
  { delay = 0, duration = 220, y = 6 }: { delay?: number; duration?: number; y?: number } = {},
): TransitionConfig {
  return {
    delay,
    duration: dur(duration),
    easing: cubicOut,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * y}px);`,
  };
}

/** Split-flap style flip for a single scoreboard character. */
export function flap(
  _node: Element,
  { delay = 0, duration = 260 }: { delay?: number; duration?: number } = {},
): TransitionConfig {
  return {
    delay,
    duration: dur(duration),
    easing: cubicOut,
    css: (t, u) => `transform: perspective(160px) rotateX(${u * -90}deg); opacity: ${0.3 + t * 0.7};`,
  };
}
