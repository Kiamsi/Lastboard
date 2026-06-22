//VERY unnecessary way to use this file
//the tauri documentation had a better way of doing it but i was stubborn

import { defineConfig } from "vite";

export default defineConfig(async () => ({

    root: "./src",

    build: {

        outDir: "../dist",
        emptyOutDir: true,
    },

    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        watch: {
            ignored: ["**/src-tauri/**"],
        },
    },
}));