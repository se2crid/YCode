import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import monacoEditorPlugin, {
  IMonacoEditorOpts,
} from "vite-plugin-monaco-editor";

// Hack because monacoEditorPlugin is not typed or exported correctly
const monacoEditorPluginDefault = (monacoEditorPlugin as any).default as (
  options: IMonacoEditorOpts
) => any;

// https://vitejs.dev/config/
const plugins = [
  react(),
  monacoEditorPluginDefault({
    languageWorkers: ["json", "editorWorkerService"],
  }),
];

export default defineConfig(async () => ({
  plugins: plugins,
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
