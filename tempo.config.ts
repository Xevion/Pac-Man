import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import type { RunContext } from "@xevion/tempo";
import { defineConfig, task } from "@xevion/tempo";

// Dev ports are sequential, with the user-facing web port first. Rolled once via
// `openssl rand`; committed as defaults in .env.example and overridable via .env.
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
const SERVER_ENV = {
  HOST: process.env.HOST || "127.0.0.1",
  PORT: SERVER_PORT,
  DATABASE_URL,
  JWT_SECRET: process.env.JWT_SECRET || "dev-insecure-jwt-secret-change-me",
  PUBLIC_BASE_URL: process.env.PUBLIC_BASE_URL || `http://localhost:${WEB_PORT}`,
};

async function container(ctx: RunContext): Promise<{ State?: string } | null> {
  const { stdout } = await ctx.capture([
    "docker", "ps", "-a", "--filter", `name=^${DB_CONTAINER}$`, "--format", "json",
  ]);
  return stdout.trim() ? JSON.parse(stdout.trim()) : null;
}

function create(ctx: RunContext): Promise<number> {
  return ctx.run([
    "docker", "run", "-d", "--name", DB_CONTAINER,
    "-e", `POSTGRES_USER=${DB_USER}`,
    "-e", `POSTGRES_PASSWORD=${DB_PASS}`,
    "-e", `POSTGRES_DB=${DB_NAME}`,
    "-p", `${DB_PORT}:5432`,
    "-v", `${DB_VOLUME}:/var/lib/postgresql/data`,
    "postgres:17",
  ]);
}

/** Ensure the Postgres container exists and is running. */
async function ensurePostgres(ctx: RunContext): Promise<void> {
  const existing = await container(ctx);
  if (!existing) {
    ctx.log(`creating Postgres container ${DB_CONTAINER} on port ${DB_PORT}`);
    await create(ctx);
    return;
  }
  if (existing.State !== "running") await ctx.run(["docker", "start", DB_CONTAINER]);
}

/** Write or replace DATABASE_URL in the root .env, creating the file if needed. */
function writeDatabaseUrl(root: string): void {
  const file = join(root, ENV_FILE);
  const current = existsSync(file) ? readFileSync(file, "utf8") : "";
  const line = `DATABASE_URL=${DATABASE_URL}`;
  writeFileSync(
    file,
    /^DATABASE_URL=.*$/m.test(current)
      ? current.replace(/^DATABASE_URL=.*$/m, line)
      : `${current.trim() ? `${current.trimEnd()}\n` : ""}${line}\n`,
  );
}

export default defineConfig({
  tasks: [
    task({
      name: "web:deps",
      cwd: "web",
      body: (ctx) => {
        if (!existsSync(join(ctx.cwd, "node_modules"))) {
          ctx.fail("web/node_modules not found -- run `bun install --cwd web` first");
        }
      },
    }),

    // Owns the whole Cargo workspace (pacman, pacman-server, pacman-common), so
    // one command reproduces the workspace-wide checks.
    task({ name: "server:format", body: "cargo fmt --all --check", tags: ["check"] }),
    task({ name: "server:format-fix", body: "cargo fmt --all", tags: ["format"] }),
    task({
      name: "server:lint",
      body: "cargo clippy --workspace --all-targets --all-features --quiet -- -D warnings",
      tags: ["check", "lint"],
    }),
    // emsdk_env.sh self-locates through $BASH_SOURCE, so it needs bash rather than
    // sh. --lib --bins, not --all-targets: this target builds panic=abort, and test
    // harnesses cannot be built with panic=abort on stable Rust.
    task({
      name: "server:lint-wasm",
      body: [
        "bash", "-c",
        "EMSDK_QUIET=1 . ./emsdk/emsdk_env.sh && cargo clippy -p pacman --target wasm32-unknown-emscripten --lib --bins --all-features --quiet -- -D warnings",
      ],
      tags: ["check", "lint"],
      requires: [{ file: "emsdk/emsdk_env.sh", hint: "clone and install emsdk" }],
    }),
    task({
      name: "server:test",
      body: "cargo nextest run --workspace --no-fail-fast",
      tags: ["test"],
      requires: [{ tool: "cargo-nextest", hint: "cargo install cargo-nextest" }],
    }),
    task({ name: "server:build", body: "cargo build -p pacman-server", tags: ["build"] }),

    task({
      name: "web:format",
      cwd: "web",
      body: "bunx prettier --check .",
      tags: ["check"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:format-fix",
      cwd: "web",
      body: "bun run format",
      tags: ["format"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:lint",
      cwd: "web",
      body: "bun run lint",
      tags: ["check", "lint"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:type-check",
      cwd: "web",
      body: "bun run check",
      tags: ["check"],
      needs: ["web:deps"],
    }),
    task({
      name: "web:test",
      cwd: "web",
      body: "bun run test",
      tags: ["test"],
      needs: ["web:deps"],
    }),

    task({
      name: "server:dev-build",
      body: "cargo build -p pacman-server",
    }),
    task({
      name: "server:dev",
      body: ["./target/debug/pacman-server"],
      tags: ["dev"],
      persistent: true,
      passthrough: true,
      env: SERVER_ENV,
      needs: ["server:dev-build"],
      after: ["db"],
      // interrupt: cargo rewrites the binary that is currently executing.
      watch: {
        paths: [
          "pacman-server/src", "pacman-common/src",
          "Cargo.toml", "Cargo.lock", "pacman-server/Cargo.toml", ENV_FILE,
        ],
        exts: [".rs", ".toml", ".lock"],
        debounce: 300,
        interrupt: true,
      },
    }),
    task({
      name: "web:dev",
      cwd: "web",
      body: ["bun", "run", "dev", "--port", WEB_PORT],
      tags: ["dev"],
      persistent: true,
      needs: ["web:deps"],
      env: { VITE_API_TARGET: `http://localhost:${SERVER_PORT}` },
    }),

    task({
      name: "db",
      tags: ["dev"],
      passthrough: true,
      // Ordering only: without docker the server falls back to in-memory SQLite.
      requires: [{ tool: "docker", hint: "the dev database needs Docker" }],
      body: async (ctx) => {
        const sub = ctx.args[0] ?? "start";

        if (sub === "rm") {
          await ctx.capture(["docker", "rm", "--force", "--volumes", DB_CONTAINER]);
          await ctx.capture(["docker", "volume", "rm", DB_VOLUME]);
          ctx.log("removed");
          return 0;
        }

        if (sub === "reset") {
          if (!(await container(ctx))) {
            await create(ctx);
          } else {
            const psql = (sql: string) =>
              ctx.run([
                "docker", "exec", DB_CONTAINER,
                "psql", "-U", DB_USER, "-d", "postgres", "-c", sql,
              ]);
            await psql(`DROP DATABASE IF EXISTS "${DB_NAME}"`);
            await psql(`CREATE DATABASE "${DB_NAME}"`);
          }
          writeDatabaseUrl(ctx.cwd);
          ctx.log(`reset (db -> localhost:${DB_PORT})`);
          return 0;
        }

        if (sub !== "start") ctx.fail(`unknown db command "${sub}" (start, reset, rm)`);
        await ensurePostgres(ctx);
        writeDatabaseUrl(ctx.cwd);
        ctx.log(`Postgres ready (db -> localhost:${DB_PORT}); DATABASE_URL written to .env`);
        return 0;
      },
    }),

    task({
      name: "docker:image",
      body: [
        "docker", "build",
        "--platform", "linux/amd64",
        "--file", "./pacman-server/Dockerfile",
        "--tag", "pacman-server", ".",
      ],
    }),
    task({
      name: "docker:run",
      needs: ["docker:image"],
      body: async (ctx) => {
        await ctx.capture(["docker", "rm", "--force", "--volumes", "pacman-server"]);
        return ctx.run([
          "docker", "run", "--rm",
          "--stop-timeout", "2",
          "--name", "pacman-server",
          "--publish", `${SERVER_PORT}:${SERVER_PORT}`,
          "--env", `PORT=${SERVER_PORT}`,
          "--env-file", ENV_FILE,
          "pacman-server",
        ]);
      },
    }),
  ],

  commands: {
    check: { description: "Run every check", tags: ["check"] },
    fmt: { description: "Apply every formatter", tags: ["format"], concurrency: 1 },
    lint: { description: "Lint every subsystem", tags: ["lint"] },
    test: { description: "Rust workspace plus web suites", tags: ["test"] },
    dev: {
      description: `web -> :${WEB_PORT}  api -> :${SERVER_PORT}  db -> :${DB_PORT}`,
      tags: ["dev"],
      exitBehavior: "first-exits",
    },
    db: {
      description: "Manage the local Postgres container (start | reset | rm)",
      tasks: ["db"],
      passthrough: true,
    },
    docker: { description: "Build and run the server image", tasks: ["docker:run"] },
  },
});
