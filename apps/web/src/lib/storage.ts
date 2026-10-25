/**
 * Whether this build may keep anything in the browser's storage. The hosted
 * pre-alpha copy (`VITE_ASTEN_SAMPLE=1`) keeps nothing: preferences live in
 * memory for the visit. Desktop and local builds keep device preferences.
 */
export function storageAllowed(): boolean {
  return import.meta.env.VITE_ASTEN_SAMPLE !== "1";
}
