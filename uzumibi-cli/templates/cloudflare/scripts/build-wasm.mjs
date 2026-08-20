import { copyFileSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const projectDir = join(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(
    readFileSync(join(projectDir, "package.json"), "utf8"),
);

const mode = process.argv[2] ?? "asyncify";
const supportedModes = new Set(["vanilla", "asyncify", "queue"]);
if (!supportedModes.has(mode)) {
    throw new Error(`Unknown build mode: ${mode}`);
}

const cliOption = process.argv.find((arg) =>
    arg.startsWith("--http-max-bytes="),
);
const cliOptionIndex = process.argv.indexOf("--http-max-bytes");
const cliValue = cliOption?.split("=", 2)[1]
    ?? (cliOptionIndex >= 0 ? process.argv[cliOptionIndex + 1] : undefined);
const configuredValue = cliValue
    ?? process.env.UZUMIBI_HTTP_MAX_BYTES
    ?? packageJson.uzumibi?.httpMaxBytes
    ?? 65536;
const httpMaxBytes = Number(configuredValue);

if (!Number.isSafeInteger(httpMaxBytes) || httpMaxBytes <= 0 || httpMaxBytes > 0x7fffffff) {
    throw new Error(
        `Invalid HTTP maximum size: ${configuredValue}. `
        + "Expected an integer between 1 and 2147483647.",
    );
}

function run(command, args) {
    const result = spawnSync(command, args, {
        cwd: projectDir,
        env: {
            ...process.env,
            UZUMIBI_HTTP_MAX_BYTES: String(httpMaxBytes),
        },
        stdio: "inherit",
    });
    if (result.error) throw result.error;
    if (result.status !== 0) process.exit(result.status ?? 1);
}

const cargoArgs = [
    "build",
    "--package", "$$PROJECT_NAME$$",
    "--target", "wasm32-unknown-unknown",
    "--release",
];

if (mode === "vanilla") {
    cargoArgs.push("--no-default-features");
} else {
    cargoArgs.push("--features", mode === "queue" ? "queue" : "enable-external");
}

console.log(`Building ${mode} Wasm with HTTP maximum ${httpMaxBytes} bytes`);
run("cargo", cargoArgs);

const source = join(
    projectDir,
    "target/wasm32-unknown-unknown/release/$$PROJECT_NAME_UNDERSCORE$$.wasm",
);
const outputName = mode === "queue"
    ? "$$PROJECT_NAME_UNDERSCORE$$_queue.wasm"
    : "$$PROJECT_NAME_UNDERSCORE$$.wasm";
const output = join(projectDir, "src", outputName);

if (mode === "vanilla") {
    copyFileSync(source, output);
} else {
    run("wasm-opt", [
        "--enable-bulk-memory",
        "--enable-nontrapping-float-to-int",
        "--asyncify",
        "-O2",
        source,
        "-o", output,
    ]);
}
