// Get API base URL from environment variable, or default to /api for same-origin requests
export const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

/**
 * Helper function to construct full API URLs
 * @param path - API endpoint path (without leading slash, e.g., "leaderboard/global")
 * @returns Full API URL
 */
export function getApiUrl(path: string): string {
  // Remove leading slash if present to avoid double slashes
  const cleanPath = path.startsWith("/") ? path.slice(1) : path;
  return `${API_BASE_URL}/${cleanPath}`;
}
