import { fileURLToPath, URL } from "node:url";

import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

/** 源码目录（统一使用正斜杠，避免 Windows 路径进入 ESM import）。 */
const sourceRoot = fileURLToPath(new URL("./src", import.meta.url)).replaceAll("\\", "/");

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const demo = mode === "demo";

  const alias = demo
    ? [
        // 在线 Demo：用浏览器内模拟后端替换 Tauri 命令层与两个插件
        { find: /^@\/lib\/tauri$/, replacement: `${sourceRoot}/lib/tauri.demo.ts` },
        { find: /^@tauri-apps\/api\/event$/, replacement: `${sourceRoot}/demo/event-shim.ts` },
        {
          find: /^@tauri-apps\/plugin-dialog$/,
          replacement: `${sourceRoot}/demo/dialog-shim.ts`,
        },
        { find: /^@\//, replacement: `${sourceRoot}/` },
      ]
    : [{ find: /^@\//, replacement: `${sourceRoot}/` }];

  return {
    // GitHub Pages 项目站点部署在 /lumos/ 下
    base: demo ? "/lumos/" : "/",
    plugins: [react(), tailwindcss()],

    resolve: { alias },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    clearScreen: false,
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
        ignored: ["**/src-tauri/**"],
      },
    },
  };
});
