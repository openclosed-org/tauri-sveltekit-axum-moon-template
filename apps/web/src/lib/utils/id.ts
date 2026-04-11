import { v7 as uuidv7 } from 'uuid';

/**
 * Generate a time-ordered UUID v7.
 *
 * UUID v7 embeds a Unix timestamp in the first 48 bits, making IDs:
 * - Sortable by creation time (useful for log correlation)
 * - More index-friendly in databases than v4
 * - RFC 9562 compliant
 */
export function newId(): string {
  return uuidv7();
}
