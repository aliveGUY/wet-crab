import init from "./pkg/runst_poc.js";

async function main() {
  const canvas = document.getElementById("webgl-canvas");

  const dpr = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();
  canvas.width = Math.round(rect.width * dpr);
  canvas.height = Math.round(rect.height * dpr);

  if (window.reRender) window.reRender();

  await init();
}

main();
