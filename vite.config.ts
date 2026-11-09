// @ts-expect-error type error without @types/node package
import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
// @ts-expect-error type error without @types/node package
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue()],

  resolve: {
    // 统一用 @ 指向 src，避免深层相对路径
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  build: {
    rollupOptions: {
      output: {
        // 把体积较大的第三方库拆成独立分块：
        // 首屏只加载 vue 框架与首页，图谱 / Markdown 引擎随懒加载页面按需加载；
        // 业务代码更新后 vendor 分块哈希不变，二次启动可直接命中缓存。
        manualChunks(id: string) {
          if (!id.includes("node_modules")) return;
          if (id.includes("@vue-flow")) return "graph";
          if (id.includes("markdown-it")) return "markdown";
          if (/node_modules[\\/](?:vue|@vue|pinia|vue-router)[\\/]/.test(id)) {
            return "vue";
          }
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
