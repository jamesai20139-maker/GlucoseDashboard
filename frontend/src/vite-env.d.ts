/// <reference types="vite/client" />

declare module '*.css';

declare module 'react' {
  export type ReactNode = any;
  export const StrictMode: any;
  export function useCallback<T extends (...args: any[]) => any>(callback: T, deps: unknown[]): T;
  export function useEffect(effect: () => void | (() => void), deps?: unknown[]): void;
  export function useMemo<T>(factory: () => T, deps: unknown[]): T;
  export function useState<T>(initialValue: T): [T, (value: T) => void];
}

declare module 'react-dom/client' {
  export function createRoot(element: Element): { render(node: unknown): void };
}

declare module 'react/jsx-runtime' {
  export function jsx(type: unknown, props: unknown): unknown;
  export function jsxs(type: unknown, props: unknown): unknown;
  export const Fragment: unknown;
}

declare namespace JSX {
  interface IntrinsicElements {
    [elementName: string]: any;
  }
}
