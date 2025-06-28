import init, { calculate_tax } from "./pkg/runst_poc.js";

async function run() {
  await init();
  console.log(calculate_tax(9000));
}

run();
