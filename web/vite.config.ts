import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig(({ command }) => {
	return {
		plugins: [react()],
		base: command === "build" ? "/seal/" : "/",
		optimizeDeps: {
			exclude: ["seal-cli"],
			include: ["monaco-editor/esm/vs/editor/editor.worker"],
		},
		server: {
			fs: {
				allow: [".."],
			},
		},
		define: {
			global: "globalThis",
		},
		worker: {
			format: "es",
		},
	};
});
