import { $ } from "bun";
import { existsSync, promises as fs } from "fs";
import { platform } from "os";
import { dirname, join, relative, resolve } from "path";
import { match, P } from "ts-pattern";

type Os =
  | { type: "linux"; wsl: boolean }
  | { type: "windows" }
  | { type: "macos" };

const os: Os = match(platform())
  .with("win32", () => ({ type: "windows" as const }))
  .with("linux", () => ({
    type: "linux" as const,
    // We detect WSL by checking for the presence of the WSLInterop file.
    // This is a semi-standard method of detecting WSL, which is more than workable for this already hacky script.
    wsl: existsSync("/proc/sys/fs/binfmt_misc/WSLInterop"),
  }))
  .with("darwin", () => ({ type: "macos" as const }))
  .otherwise(() => {
    throw new Error(`Unsupported platform: ${platform()}`);
  });

function log(msg: string) {
  console.log(`[web.build] ${msg}`);
}

/**
 * Build the application with Emscripten, generate the CSS, and copy the files into 'dist'.
 *
 * @param release - Whether to build in release mode.
 * @param env - The environment variables to inject into build commands.
 */
async function build(release: boolean, env: Record<string, string>) {
  log(
    `Building for 'wasm32-unknown-emscripten' for ${
      release ? "release" : "debug"
    }`
  );
  await $`cargo build --target=wasm32-unknown-emscripten ${
    release ? "--release" : ""
  }`.env(env);

  log("Generating CSS");
  await $`pnpx postcss-cli ./assets/site/styles.scss -o ./assets/site/build.css`;

  const buildType = release ? "release" : "debug";
  const siteFolder = resolve("assets/site");
  const outputFolder = resolve(`target/wasm32-unknown-emscripten/${buildType}`);
  const dist = resolve("dist");

  // The files to copy into 'dist'
  const files = [
    ...["index.html", "favicon.ico", "build.css"].map((file) => ({
      src: join(siteFolder, file),
      dest: join(dist, file),
      optional: false,
    })),
    ...["pacman.wasm", "pacman.js", "deps/pacman.data"].map((file) => ({
      src: join(outputFolder, file),
      dest: join(dist, file.split("/").pop() || file),
      optional: false,
    })),
    {
      src: join(outputFolder, "pacman.wasm.map"),
      dest: join(dist, "pacman.wasm.map"),
      optional: true,
    },
  ];

  // Create required destination folders
  await Promise.all(
    // Get the dirname of files, remove duplicates
    [...new Set(files.map(({ dest }) => dirname(dest)))]
      // Create the folders
      .map(async (dir) => {
        // If the folder doesn't exist, create it
        if (!(await fs.exists(dir))) {
          log(`Creating folder ${dir}`);
          await fs.mkdir(dir, { recursive: true });
        }
      })
  );

  // Copy the files to the dist folder
  log("Copying files into dist");
  await Promise.all(
    files.map(async ({ optional, src, dest }) => {
      match({ optional, exists: await fs.exists(src) })
        // If optional and doesn't exist, skip
        .with({ optional: true, exists: false }, () => {
          log(
            `Optional file ${os.type === "windows" ? "\\" : "/"}${relative(
              process.cwd(),
              src
            )} does not exist, skipping...`
          );
        })
        // If not optional and doesn't exist, throw an error
        .with({ optional: false, exists: false }, () => {
          throw new Error(`Required file ${src} does not exist`);
        })
        // Otherwise, copy the file
        .otherwise(async () => await fs.copyFile(src, dest));
    })
  );
}

/**
 * Checks to see if the Emscripten SDK is activated for a Windows or *nix machine by looking for a .exe file and the equivalent file on Linux/macOS. Returns both results for handling.
 * @param emsdkDir - The directory containing the Emscripten SDK.
 * @returns A record of environment variables.
 */
async function checkEmsdkType(
  emsdkDir: string
): Promise<{ windows: boolean; nix: boolean }> {
  const binary = resolve(join(emsdkDir, "upstream", "bin", "clang"));

  return {
    windows: await fs.exists(binary + ".exe"),
    nix: await fs.exists(binary),
  };
}

/**
 * Activate the Emscripten SDK environment variables.
 * Technically, this doesn't actaully activate the environment variables for the current shell,
 * it just runs the environment sourcing script and returns the environment variables for future command invocations.
 * @param emsdkDir - The directory containing the Emscripten SDK.
 * @returns A record of environment variables.
 */
async function activateEmsdk(
  emsdkDir: string
): Promise<{ vars: Record<string, string> } | { err: string }> {
  // Determine the environment script to use based on the OS
  const envScript = match(os)
    .with({ type: "windows" }, () => join(emsdkDir, "emsdk_env.bat"))
    .with({ type: P.union("linux", "macos") }, () =>
      join(emsdkDir, "emsdk_env.sh")
    )
    .exhaustive();

  // Run the environment script and capture the output
  const { stdout, stderr, exitCode } = await match(os)
    .with({ type: "windows" }, () =>
      // run the script, ignore it's output ('>nul'), then print the environment variables ('set')
      $`cmd /c "${envScript} >nul && set"`.quiet()
    )
    .with({ type: P.union("linux", "macos") }, () =>
      // run the script with bash, ignore it's output ('> /dev/null'), then print the environment variables ('env')
      $`bash -c "source '${envScript}' && env"`.quiet()
    )
    .exhaustive();

  if (exitCode !== 0) {
    return { err: stderr.toString() };
  }

  // Parse the output into a record of environment variables
  const vars = Object.fromEntries(
    stdout
      .toString()
      .split(os.type === "windows" ? /\r?\n/ : "\n") // Split output into lines, handling Windows CRLF vs *nix LF
      .map((line) => line.split("=", 2)) // Parse each line as KEY=VALUE (limit to 2 parts)
      .filter(([k, v]) => k && v) // Keep only valid key-value pairs (both parts exist)
  );

  return { vars };
}

async function main() {
  // Print the OS detected
  log(
    "OS Detected: " +
      match(os)
        .with({ type: "windows" }, () => "Windows")
        .with({ type: "linux" }, ({ wsl: isWsl }) =>
          isWsl ? "Linux (via WSL)" : "Linux"
        )
        .with({ type: "macos" }, () => "macOS")
        .exhaustive()
  );

  const release = process.env.RELEASE !== "0";
  const emsdkDir = resolve("./emsdk");
  // Ensure the emsdk directory exists before attempting to activate or use it
  if (!(await fs.exists(emsdkDir))) {
    log(
      `Emscripten SDK directory not found at ${emsdkDir}. Please install or clone 'emsdk' and try again.`
    );
    process.exit(1);
  }
  const vars = match(await activateEmsdk(emsdkDir)) // result handling
    .with({ vars: P.select() }, (vars) => vars)
    .with({ err: P.any }, ({ err }) => {
      log("Error activating Emscripten SDK: " + err);
      process.exit(1);
    })
    .exhaustive();

  // Check if the Emscripten SDK is activated/installed properly for the current OS
  match({
    os: os,
    ...(await checkEmsdkType(emsdkDir)),
  })
    // If the Emscripten SDK is not activated/installed properly, exit with an error
    .with(
      {
        nix: false,
        windows: false,
      },
      () => {
        log(
          "Emscripten SDK does not appear to be activated/installed properly."
        );
        process.exit(1);
      }
    )
    // If the Emscripten SDK is activated for Windows, but is currently running on a *nix OS, exit with an error
    .with(
      {
        nix: false,
        windows: true,
        os: { type: P.not("windows") },
      },
      () => {
        log(
          "Emscripten SDK appears to be activated for Windows, but is currently running on a *nix OS."
        );
        process.exit(1);
      }
    )
    // If the Emscripten SDK is activated for *nix, but is currently running on a Windows OS, exit with an error
    .with(
      {
        nix: true,
        windows: false,
        os: { type: "windows" },
      },
      () => {
        log(
          "Emscripten SDK appears to be activated for *nix, but is currently running on a Windows OS."
        );
        process.exit(1);
      }
    );

  // Build the application
  await build(release, vars);
}

/**
 * Main entry point.
 */
main().catch((err) => {
  console.error("[web.build] Error:", err);
  process.exit(1);
});
