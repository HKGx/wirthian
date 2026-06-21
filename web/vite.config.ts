import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import wasm from "vite-plugin-wasm";
import { lezer } from "@lezer/generator/rollup";

// https://vite.dev/config/
export default defineConfig({
    plugins: [svelte(), wasm(), lezer()],
    server: {
        fs: { allow: [".."] },
    },
});
