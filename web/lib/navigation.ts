import { useSyncExternalStore } from "react";

type Listener = (pendingUrl: string | null) => void;

let pendingUrl: string | null = null;
const listeners = new Set<Listener>();

export function setPendingNavigation(url: string | null) {
  pendingUrl = url;
  listeners.forEach((listener) => listener(pendingUrl));
}

export function getPendingNavigation(): string | null {
  return pendingUrl;
}

export function subscribeToPendingNavigation(listener: Listener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function usePendingNavigation(): string | null {
  return useSyncExternalStore(subscribeToPendingNavigation, getPendingNavigation, () => null);
}
