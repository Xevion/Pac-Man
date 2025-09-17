import { $ } from "bun";
import { readFileSync, writeFileSync, existsSync } from "fs";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { createInterface } from "readline";

// Helper function to get user input
async function getUserChoice(
  prompt: string,
  choices: string[],
  defaultIndex: number = 1
): Promise<string> {
  // Check if we're in an interactive TTY
  if (!process.stdin.isTTY) {
    console.log(
      "Non-interactive environment detected; selecting default option " +
        defaultIndex
    );
    return String(defaultIndex);
  }

  console.log(prompt);
  choices.forEach((choice, index) => {
    console.log(`${index + 1}. ${choice}`);
  });

  // Use readline for interactive input
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  return new Promise((resolve) => {
    const askForChoice = () => {
      rl.question("Enter your choice (1-3): ", (answer) => {
        const choice = answer.trim();
        if (["1", "2", "3"].includes(choice)) {
          rl.close();
          resolve(choice);
        } else {
          console.log("Invalid choice. Please enter 1, 2, or 3.");
          askForChoice();
        }
      });
    };
    askForChoice();
  });
}

// Get repository root path from script location
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const repoRoot = join(__dirname, "..");
const envPath = join(repoRoot, "pacman-server", ".env");

console.log("Checking for .env file...");

// Check if .env file exists and read it
let envContent = "";
let envLines: string[] = [];
let databaseUrlLine = -1;
let databaseUrlValue = "";

if (existsSync(envPath)) {
  console.log("Found .env file, reading...");
  envContent = readFileSync(envPath, "utf-8");
  envLines = envContent.split("\n");

  // Parse .env file for DATABASE_URL
  for (let i = 0; i < envLines.length; i++) {
    const line = envLines[i].trim();
    if (line.match(/^[A-Z_][A-Z0-9_]*=.*$/)) {
      if (line.startsWith("DATABASE_URL=")) {
        databaseUrlLine = i;
        databaseUrlValue = line.substring(13); // Remove "DATABASE_URL="
        break;
      }
    }
  }
} else {
  console.log("No .env file found, will create one");
}

// Determine user's choice
let userChoice = "2"; // Default to print

if (databaseUrlLine !== -1) {
  console.log(`Found existing DATABASE_URL: ${databaseUrlValue}`);
  userChoice = await getUserChoice("\nChoose an action:", [
    "Quit",
    "Print (create container, print DATABASE_URL)",
    "Replace (update DATABASE_URL in .env)",
  ]);

  if (userChoice === "1") {
    console.log("Exiting...");
    process.exit(0);
  }
} else {
  console.log("No existing DATABASE_URL found");

  // Ask what to do when no .env file or DATABASE_URL exists
  if (!existsSync(envPath)) {
    userChoice = await getUserChoice(
      "\nNo .env file found. What would you like to do?",
      [
        "Print (create container, print DATABASE_URL)",
        "Create .env file and add DATABASE_URL",
        "Quit",
      ]
    );

    if (userChoice === "3") {
      console.log("Exiting...");
      process.exit(0);
    }
  } else {
    console.log("Will add DATABASE_URL to existing .env file");
  }
}

// Check if container exists
console.log("Checking for existing container...");
const containerExists =
  await $`docker ps -a --filter name=pacman-server-postgres --format "{{.Names}}"`
    .text()
    .then((names) => names.trim() === "pacman-server-postgres")
    .catch(() => false);

let shouldReplaceContainer = false;

if (containerExists) {
  console.log("Container already exists");

  // Always ask what to do if container exists
  const replaceChoice = await getUserChoice(
    "\nContainer exists. What would you like to do?",
    ["Use existing container", "Replace container (remove and create new)"],
    1
  );
  shouldReplaceContainer = replaceChoice === "2";

  if (shouldReplaceContainer) {
    console.log("Removing existing container...");
    await $`docker rm --force --volumes pacman-server-postgres`;
  } else {
    console.log("Using existing container");
  }
}

// Create container if needed
if (!containerExists || shouldReplaceContainer) {
  console.log("Creating PostgreSQL container...");
  await $`docker run --detach --name pacman-server-postgres --publish 5432:5432 --env POSTGRES_USER=postgres --env POSTGRES_PASSWORD=postgres --env POSTGRES_DB=pacman-server postgres:17`;
}

// Format DATABASE_URL
const databaseUrl =
  "postgresql://postgres:postgres@localhost:5432/pacman-server";

// Handle the final action based on user choice
if (userChoice === "2") {
  // Print option
  console.log(`\nDATABASE_URL=${databaseUrl}`);
} else if (
  userChoice === "3" ||
  (databaseUrlLine === -1 && userChoice === "2")
) {
  // Replace or add to .env file
  if (databaseUrlLine !== -1) {
    // Replace existing line
    console.log("Updating DATABASE_URL in .env file...");
    envLines[databaseUrlLine] = `DATABASE_URL=${databaseUrl}`;
    writeFileSync(envPath, envLines.join("\n"));
    console.log("Updated .env file");
  } else {
    // Add new line
    console.log("Adding DATABASE_URL to .env file...");
    const newContent =
      envContent +
      (envContent.endsWith("\n") ? "" : "\n") +
      `DATABASE_URL=${databaseUrl}\n`;
    writeFileSync(envPath, newContent);
    console.log("Added to .env file");
  }
}
