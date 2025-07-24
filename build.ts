import { $ } from "bun";

// This is a bun script, run with `bun run build.ts`

import * as path from "path";
import * as fs from "fs/promises";

async function clean() {
  console.log("Cleaning...");
  await $`cargo clean`;
  await $`rm -rf ./dist/`;
  console.log("Cleaned...");
}

async function setupEmscripten() {
  const emsdkDir = "./emsdk";
  const emsdkExists = await fs
    .access(emsdkDir)
    .then(() => true)
    .catch(() => false);

  if (!emsdkExists) {
    console.log("Cloning Emscripten SDK...");
    await $`git clone https://github.com/emscripten-core/emsdk.git`;
  } else {
    console.log("Emscripten SDK already exists, skipping clone.");
  }

  const emscriptenToolchainPath = path.join(emsdkDir, "upstream", "emscripten");
  const toolchainInstalled = await fs
    .access(emscriptenToolchainPath)
    .then(() => true)
    .catch(() => false);

  if (!toolchainInstalled) {
    console.log("Installing Emscripten toolchain...");
    await $`./emsdk/emsdk install 3.1.43`;
  } else {
    console.log(
      "Emscripten toolchain 3.1.43 already installed, skipping install."
    );
  }

  console.log("Activating Emscripten...");
  await $`./emsdk/emsdk activate 3.1.43`;
  console.log("Emscripten activated.");

  // Set EMSDK environment variable for subsequent commands
  process.env.EMSDK = path.resolve(emsdkDir);

  const emsdkPython = path.join(path.resolve(emsdkDir), "python");
  const emsdkNode = path.join(path.resolve(emsdkDir), "node", "16.20.0_64bit"); // Adjust node version if needed
  const emsdkBin = path.join(path.resolve(emsdkDir), "upstream", "emscripten");
  process.env.PATH = `${emsdkPython}:${emsdkNode}:${emsdkBin}:${process.env.PATH}`;
}

async function buildWeb(release: boolean) {
  console.log("Building WASM with Emscripten...");
  const rustcFlags = [
    "-C",
    "link-arg=--preload-file",
    "-C",
    "link-arg=assets",
  ].join(" ");

  if (release) {
    await $`env RUSTFLAGS=${rustcFlags} cargo build --target=wasm32-unknown-emscripten --release`;
  } else {
    await $`env RUSTFLAGS=${rustcFlags} cargo build --target=wasm32-unknown-emscripten`;
  }

  console.log("Generating CSS...");
  await $`pnpx postcss-cli ./assets/site/styles.scss -o ./assets/site/build.css`;

  console.log("Copying WASM files...");
  const buildType = release ? "release" : "debug";
  const outputFolder = `target/wasm32-unknown-emscripten/${buildType}`;
  await $`mkdir -p dist`;
  await $`cp assets/site/index.html dist`;
  await $`cp assets/site/*.woff* dist`;
  await $`cp assets/site/build.css dist`;
  await $`cp assets/site/favicon.ico dist`;
  await $`cp ${outputFolder}/pacman.wasm dist`;
  await $`cp ${outputFolder}/pacman.js dist`;

  // Check if .data file exists before copying
  try {
    await fs.access(`${outputFolder}/pacman.data`);
    await $`cp ${outputFolder}/pacman.data dist`;
  } catch (e) {
    console.log("No pacman.data file found, skipping copy.");
  }

  // Check if .map file exists before copying
  try {
    await fs.access(`${outputFolder}/pacman.wasm.map`);
    await $`cp ${outputFolder}/pacman.wasm.map dist`;
  } catch (e) {
    console.log("No pacman.wasm.map file found, skipping copy.");
  }

  console.log("WASM files copied.");
}

async function serve() {
  console.log("Serving WASM with Emscripten...");
  await $`python3 -m http.server -d ./dist/ 8080`;
}

async function main() {
  const args = process.argv.slice(2);

  let release = false;
  let serveFiles = false;
  let skipEmscriptenSetup = false;
  let cleanProject = false;
  let target = "web"; // Default target

  for (const arg of args) {
    switch (arg) {
      case "-r":
        release = true;
        break;
      case "-s":
        serveFiles = true;
        break;
      case "-e":
        skipEmscriptenSetup = true;
        break;
      case "-c":
        cleanProject = true;
        break;
      case "--target=linux":
        target = "linux";
        break;
      case "--target=windows":
        target = "windows";
        break;
      case "--target=web":
        target = "web";
        break;
      case "-h":
      case "--help":
        console.log(`
Usage: ts-node build.ts [options]

Options:
  -r              Build in release mode
  -s              Serve the WASM files once built (for web target)
  -e              Skip EMSDK setup (GitHub workflow only)
  -c              Clean the target/dist directory
  --target=[web|linux|windows] Specify target platform (default: web)
  -h, --help      Show this help message
        `);
        return;
    }
  }

  if (cleanProject) {
    await clean();
  }

  if (!skipEmscriptenSetup && target === "web") {
    await setupEmscripten();
  }

  switch (target) {
    case "web":
      await buildWeb(release);
      if (serveFiles) {
        await serve();
      }
      break;
    case "linux":
      console.log("Building for Linux...");
      if (release) {
        await $`cargo build --release`;
      } else {
        await $`cargo build`;
      }
      console.log("Linux build complete.");
      break;
    case "windows":
      console.log("Building for Windows...");
      if (release) {
        await $`cargo build --release --target=x86_64-pc-windows-msvc`; // Assuming MSVC toolchain
      } else {
        await $`cargo build --target=x86_64-pc-windows-msvc`;
      }
      console.log("Windows build complete.");
      break;
    default:
      console.error("Invalid target specified.");
      process.exit(1);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
