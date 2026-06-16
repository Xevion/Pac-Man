import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { defineConfig, runners } from "@xevion/tempo";
import { c } from "@xevion/tempo/fmt";
import { hasTool, run, runPiped } from "@xevion/tempo/proc";

// Dev ports are sequential, with the user-facing web port first. Rolled once via
// `openssl rand`; committed as defaults in .env.example and overridable via .env
// (loaded by the Justfile's `set dotenv-load`).
const WEB_PORT = process.env.PACMAN_WEB_PORT || "42565";
const SERVER_PORT = process.env.PACMAN_SERVER_PORT || String(Number(WEB_PORT) + 1);
const DB_PORT = process.env.PACMAN_DB_PORT || String(Number(WEB_PORT) + 2);

const DB_USER = "postgres";
const DB_PASS = "postgres";
const DB_NAME = "pacman";
const DB_CONTAINER = "pacman-postgres";
const DB_VOLUME = "pacman-postgres-data";
const ENV_FILE = ".env";

// The server reads pure env vars (figment Env::raw), so dev values are injected
// directly into the spawned process. Defaults let `tempo dev` boot with no .env.
const DATABASE_URL =
  process.env.DATABASE_URL || `postgresql://${DB_USER}:${DB_PASS}@127.0.0.1:${DB_PORT}/${DB_NAME}`;
const JWT_SECRET = process.env.JWT_SECRET || "dev-insecure-jwt-secret-change-me";
const PUBLIC_BASE_URL = process.env.PUBLIC_BASE_URL || `http://localhost:${WEB_PORT}`;
const HOST = process.env.HOST || "127.0.0.1";

/** Parsed `docker ps -a` row for the dev Postgres container, or null if absent. */
function getContainer(): { State?: string } | null {
  const out = runPiped([
    "docker", "ps", "-a", "--filter", `name=^${DB_CONTAINER}$`, "--format", "json",
  ]).stdout.trim();
  return out ? JSON.parse(out) : null;
}

function createContainer(): void {
  run([
    "docker", "run", "-d", "--name", DB_CONTAINER,
    "-e", `POSTGRES_USER=${DB_USER}`,
    "-e", `POSTGRES_PASSWORD=${DB_PASS}`,
    "-e", `POSTGRES_DB=${DB_NAME}`,
    "-p", `${DB_PORT}:5432`,
    "-v", `${DB_VOLUME}:/var/lib/postgresql/data`,
    "postgres:17",
  ]);
}

/** Ensure the Postgres container exists and is running. Returns false if docker is absent. */
function ensurePostgres(log: (msg: string) => void): boolean {
  if (!hasTool("docker")) return false;
  const container = getContainer();
  if (!container) {
    log(`creating Postgres container ${DB_CONTAINER} on port ${DB_PORT}`);
    createContainer();
  } else if (container.State !== "running") {
    run(["docker", "start", DB_CONTAINER]);
  }
  return true;
}

/** Write/replace DATABASE_URL in the root .env, creating the file if needed. */
function writeDatabaseUrl(): void {
  let content = "";
  try {
    content = readFileSync(ENV_FILE, "utf8");
  } catch {
    // no .env yet -- start from empty
  }
  if (/^DATABASE_URL=.*$/m.test(content)) {
    content = content.replace(/^DATABASE_URL=.*$/m, `DATABASE_URL=${DATABASE_URL}`);
  } else {
    content = (content.trim() ? `${content.trimEnd()}\n` : "") + `DATABASE_URL=${DATABASE_URL}\n`;
  }
  writeFileSync(ENV_FILE, content);
}

export default defineConfig({
  subsystems: {
    // Owns the whole Cargo workspace (pacman, pacman-server, pacman-common) so a single
    // command reproduces the old workspace-wide checks. The dev process runs the server.
    server: {
      aliases: ["s", "server", "back", "be", "rust", "rs", "game", "g"],
      requires: ["cargo"],
      commands: {
        "format-check": "cargo fmt --all --check",
        "format-apply": "cargo fmt --all",
        lint: "cargo clippy --workspace --all-targets --all-features --quiet -- -D warnings",
        // tempo runs string commands via dash's `sh -c`, but emsdk_env.sh self-locates
        // through $BASH_SOURCE (unset in dash), so source it under bash. This puts emcc
        // on PATH for the wasm build scripts; EMSDK_QUIET silences its setup banner.
        "lint-wasm":
          "bash -c 'EMSDK_QUIET=1 . ./emsdk/emsdk_env.sh && cargo clippy -p pacman --target wasm32-unknown-emscripten --all-targets --all-features --quiet -- -D warnings'",
        test: "cargo nextest run --workspace --no-fail-fast",
        build: "cargo build -p pacman-server",
      },
      autoFix: { "format-check": "format-apply" },
    },
    frontend: {
      aliases: ["f", "front", "web", "fe"],
      cwd: "web",
      requires: ["bun"],
      commands: {
        "format-check": "bunx prettier --check .",
        "format-apply": "bun run format",
        lint: "bun run lint",
        "type-check": "bun run check",
        test: "bun run test",
      },
      autoFix: { "format-check": "format-apply" },
    },
  },
  hooks: {
    "before:dev": (ctx) => {
      if (ctx.targets.has("frontend") && !existsSync("web/node_modules")) {
        ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
      }
      if (ctx.targets.has("server")) {
        if (!hasTool("cargo")) ctx.fail("cargo not found -- install the Rust toolchain first");
        if (!ensurePostgres((msg) => ctx.logger.info(msg))) {
          ctx.logger.warn(
            "docker not found -- starting server without Postgres (SQLite in-memory fallback)",
          );
        }
      }
      ctx.logger.info(
        c.catGreen(
          `web -> http://localhost:${WEB_PORT}   api -> http://localhost:${SERVER_PORT}   db -> localhost:${DB_PORT}`,
        ),
      );
    },
  },
  dev: {
    exitBehavior: "first-exits",
    processes: {
      server: {
        type: "managed",
        watch: {
          dirs: ["pacman-server/src", "pacman-common/src"],
          exts: [".rs"],
          extraPaths: ["Cargo.toml", "Cargo.lock", "pacman-server/Cargo.toml", ENV_FILE],
          debounce: 300,
        },
        build: { cmd: "cargo build -p pacman-server", verbose: false },
        run: { cmd: "./target/debug/pacman-server", passthrough: true },
        interrupt: true,
        env: { HOST, PORT: SERVER_PORT, DATABASE_URL, JWT_SECRET, PUBLIC_BASE_URL },
      },
      frontend: {
        type: "unmanaged",
        cmd: ["bun", "run", "dev", "--port", WEB_PORT],
        cwd: "web",
        env: { VITE_API_TARGET: `http://localhost:${SERVER_PORT}` },
      },
    },
  },
  commands: {
    check: runners.check({
      autoFixStrategy: "fix-first",
      // Checks only -- tests and the server build run via `tempo test` / `tempo dev`.
      exclude: ["server:test", "server:build", "frontend:test"],
    }),
    fmt: runners.sequential("format-apply", {
      description: "Sequential per-subsystem formatting",
      autoFixFallback: true,
    }),
    lint: runners.sequential("lint", { description: "Sequential per-subsystem linting" }),
    test: runners.sequential("test", { description: "Run the Rust workspace + web test suites" }),
    dev: runners.dev(),
    "pre-commit": runners.preCommit(),
    db: {
      description: "Manage the local Postgres container (start | reset | rm)",
      parameters: ["[subcommand]"],
      run: async (ctx) => {
        const sub = ctx.args[0] || "start";
        if (!hasTool("docker")) {
          console.error(c.catRed("docker not found -- install Docker to use the dev database"));
          return 1;
        }

        if (sub === "rm") {
          ctx.runPiped(["docker", "rm", "--force", "--volumes", DB_CONTAINER]);
          ctx.runPiped(["docker", "volume", "rm", DB_VOLUME]);
          console.error(c.catGreen("removed"));
          return 0;
        }

        if (sub === "reset") {
          if (!getContainer()) {
            createContainer();
          } else {
            const psql = (sql: string) =>
              ctx.run([
                "docker", "exec", DB_CONTAINER, "psql", "-U", DB_USER, "-d", "postgres", "-c", sql,
              ]);
            psql(`DROP DATABASE IF EXISTS "${DB_NAME}"`);
            psql(`CREATE DATABASE "${DB_NAME}"`);
          }
          writeDatabaseUrl();
          console.error(c.catGreen(`reset (db -> localhost:${DB_PORT})`));
          return 0;
        }

        // Default: start
        ensurePostgres((msg) => console.error(c.catBlue(msg)));
        writeDatabaseUrl();
        console.error(c.catGreen(`Postgres ready (db -> localhost:${DB_PORT}), DATABASE_URL written to .env`));
        return 0;
      },
    },
    docker: {
      image: {
        description: "Build the server Docker image",
        run: async (ctx) => {
          ctx.run([
            "docker", "build",
            "--platform", "linux/amd64",
            "--file", "./pacman-server/Dockerfile",
            "--tag", "pacman-server",
            ".",
          ]);
          return 0;
        },
      },
      run: {
        description: "Build and run the server in a Docker container",
        run: async (ctx) => {
          ctx.run([
            "docker", "build",
            "--platform", "linux/amd64",
            "--file", "./pacman-server/Dockerfile",
            "--tag", "pacman-server",
            ".",
          ]);
          ctx.runPiped(["docker", "rm", "--force", "--volumes", "pacman-server"]);
          ctx.run([
            "docker", "run", "--rm",
            "--stop-timeout", "2",
            "--name", "pacman-server",
            "--publish", `${SERVER_PORT}:${SERVER_PORT}`,
            "--env", `PORT=${SERVER_PORT}`,
            "--env-file", ENV_FILE,
            "pacman-server",
          ]);
          return 0;
        },
      },
    },
  },
});
