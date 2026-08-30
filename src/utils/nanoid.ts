// ============================================================
// Tatpar — Tiny nanoid utility
// Avoids pulling in the full nanoid package for simple IDs
// ============================================================

const CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/**
 * Generate a random alphanumeric ID string.
 * @param length Default: 12
 */
export function nanoid(length = 12): string {
  let result = "";
  for (let i = 0; i < length; i++) {
    result += CHARS.charAt(Math.floor(Math.random() * CHARS.length));
  }
  return result;
}
