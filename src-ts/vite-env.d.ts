/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_LOCAL_ADDRESS: string
  readonly VITE_PORT: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}