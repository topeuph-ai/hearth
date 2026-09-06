import { defineConfig } from "vite";

export default defineConfig({
  // Packaged into a .webhapp and served from a file root, so assets must be
  // referenced relatively rather than from /.
  base: "",
  build: { outDir: "dist", emptyOutDir: true },
});
