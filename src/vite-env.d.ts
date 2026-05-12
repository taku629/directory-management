/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Lemon Squeezy checkout URL for the Pro product. */
  readonly VITE_BUY_URL?: string;
  /** Lemon Squeezy checkout URL for the Team product. */
  readonly VITE_TEAM_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
