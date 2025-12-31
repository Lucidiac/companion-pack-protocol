// src/vite-plugin.ts
function companionPack(options) {
  const { packId, packName, entry = "index.ts", outDir = "dist" } = options;
  return {
    name: "companion-pack",
    config: () => ({
      build: {
        lib: {
          entry,
          name: packName,
          fileName: () => "frontend.js",
          formats: ["iife"]
        },
        rollupOptions: {
          external: ["react", "react-dom"],
          output: {
            globals: {
              react: "React",
              "react-dom": "ReactDOM"
            },
            // Self-registration footer
            footer: `
if (typeof window !== 'undefined') {
  if (!window.__COMPANION_PACKS__) window.__COMPANION_PACKS__ = {};
  window.__COMPANION_PACKS__['${packId}'] = ${packName}.default;
}
            `.trim()
          }
        },
        outDir,
        minify: true,
        sourcemap: false
      }
    })
  };
}
var vite_plugin_default = companionPack;

export {
  companionPack,
  vite_plugin_default
};
