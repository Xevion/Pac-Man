import type { Plugin, ResolvedConfig } from 'vite';
import type { Font, FontCollection } from 'fontkit';
import * as fontkit from 'fontkit';
// @ts-expect-error subset-font has no type definitions
import subsetFont from 'subset-font';
import { createHash } from 'node:crypto';

function isFont(font: Font | FontCollection): font is Font {
	return 'glyphForCodePoint' in font;
}
import { readFile, writeFile, mkdir, copyFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import path from 'node:path';
import { normalizePath } from 'vite';

// ============================================================================
// Types
// ============================================================================

export interface FontSubsetSource {
	source: string;
	whitelist: string;
	weight?: number;
	style?: 'normal' | 'italic' | 'oblique';
	family?: string;
}

export interface FontSubsetConfig {
	fonts: FontSubsetSource[];
	outputDir?: string;
	cssOutputPath?: string;
	cacheDir?: string;
	skipOnMissingDeps?: boolean;
}

interface FontMetadata {
	family: string;
	weight: number;
	style: 'normal' | 'italic' | 'oblique';
	postscriptName: string;
	isVariable: boolean;
}

interface UnicodeRange {
	cssRange: string;
	comment: string;
}

interface CacheEntry {
	sourceHash: string;
	outputPath: string;
	metadata: FontMetadata;
	unicodeRange: UnicodeRange;
	timestamp: number;
}

interface FontFaceDescriptor {
	family: string;
	weight: number;
	style: string;
	fontPath: string;
	unicodeRange: UnicodeRange;
	originalSource: string;
}

// ============================================================================
// Logging
// ============================================================================

function logInfo(message: string): void {
	console.log(`[vite-plugin-font-subset] ${message}`);
}

function logWarning(message: string): void {
	console.warn(`[vite-plugin-font-subset] WARNING: ${message}`);
}

// ============================================================================
// Path Resolution
// ============================================================================

class PathResolver {
	constructor(private viteConfig: ResolvedConfig) {}

	resolveSource(sourcePath: string): string {
		if (sourcePath.startsWith('@fontsource/')) {
			return normalizePath(path.resolve(this.viteConfig.root, 'node_modules', sourcePath));
		}

		if (path.isAbsolute(sourcePath)) {
			return normalizePath(sourcePath);
		}

		return normalizePath(path.resolve(this.viteConfig.root, sourcePath));
	}

	resolveOutputDir(configuredPath: string): string {
		return normalizePath(path.resolve(this.viteConfig.root, configuredPath));
	}

	resolveCssPath(configuredPath: string): string {
		return normalizePath(path.resolve(this.viteConfig.root, configuredPath));
	}

	resolveCacheDir(configuredPath: string): string {
		return normalizePath(path.resolve(this.viteConfig.root, configuredPath));
	}
}

// ============================================================================
// Configuration Validation
// ============================================================================

function validateConfig(config: FontSubsetConfig): void {
	if (!config.fonts || config.fonts.length === 0) {
		throw new Error('Font subset config must have at least one font');
	}

	for (const [index, font] of config.fonts.entries()) {
		if (!font.source) {
			throw new Error(`Font config [${index}]: 'source' is required`);
		}
		if (!font.whitelist || font.whitelist.length === 0) {
			throw new Error(`Font config [${index}]: 'whitelist' must contain at least one character`);
		}
		if (font.weight && (font.weight < 100 || font.weight > 900)) {
			throw new Error(`Font config [${index}]: 'weight' must be between 100 and 900`);
		}
	}
}

// ============================================================================
// Dependency Checking
// ============================================================================

async function checkDependencies(): Promise<void> {
	const required = ['fontkit', 'subset-font'];
	const missing: string[] = [];

	for (const dep of required) {
		try {
			await import(dep);
		} catch {
			missing.push(dep);
		}
	}

	if (missing.length > 0) {
		throw new Error(
			`Missing required dependencies: ${missing.join(', ')}\n` +
				`Install with: bun add -d fontkit subset-font @types/fontkit`
		);
	}
}

// ============================================================================
// Font Metadata Extraction
// ============================================================================

function inferStyle(
	subfamilyName: string | undefined,
	italicAngle: number
): 'normal' | 'italic' | 'oblique' {
	const name = (subfamilyName || '').toLowerCase();
	if (name.includes('italic')) return 'italic';
	if (name.includes('oblique')) return 'oblique';
	if (italicAngle !== 0) return 'italic';
	return 'normal';
}

function inferWeight(subfamilyName: string | undefined): number {
	const name = (subfamilyName || '').toLowerCase();

	const weightMap: Record<string, number> = {
		thin: 100,
		hairline: 100,
		'extra light': 200,
		'ultra light': 200,
		light: 300,
		regular: 400,
		normal: 400,
		medium: 500,
		'semi bold': 600,
		'demi bold': 600,
		bold: 700,
		'extra bold': 800,
		'ultra bold': 800,
		black: 900,
		heavy: 900
	};

	for (const [key, value] of Object.entries(weightMap)) {
		if (name.includes(key)) {
			return value;
		}
	}

	return 400;
}

async function extractFontMetadata(
	fontPath: string,
	overrides?: { family?: string; weight?: number; style?: string }
): Promise<FontMetadata> {
	const fontOrCollection = fontkit.openSync(fontPath);
	if (!isFont(fontOrCollection)) {
		throw new Error(`Font collections are not supported: ${fontPath}`);
	}
	const font = fontOrCollection;

	const isVariable = font.variationAxes && Object.keys(font.variationAxes).length > 0;

	// Extract family name using OpenType name table priority
	let family: string;
	let familySource: string;

	if (overrides?.family) {
		family = overrides.family;
		familySource = 'config override';
	} else {
		// OpenType name table IDs:
		// ID 16 = Typographic/Preferred Family (base family without weight/style)
		// ID 1 = Font Family (may include weight/style for compatibility)
		// eslint-disable-next-line @typescript-eslint/no-explicit-any
		const nameTable = (font as any).name;
		const preferredFamily = nameTable?.records?.preferredFamily?.en;
		const fontFamily = nameTable?.records?.fontFamily?.en;

		if (preferredFamily) {
			family = preferredFamily;
			familySource = 'Name ID 16 (Typographic Family)';
		} else if (fontFamily) {
			family = fontFamily;
			familySource = 'Name ID 1 (Font Family)';
		} else {
			family = font.familyName;
			familySource = 'familyName property';
		}
	}

	const style =
		(overrides?.style as 'normal' | 'italic' | 'oblique') ||
		inferStyle(font.subfamilyName, font.italicAngle);

	let weight: number;
	if (overrides?.weight) {
		weight = overrides.weight;
	} else if (isVariable) {
		throw new Error(
			`Variable font detected: ${fontPath}\n` +
				`Variable fonts require explicit weight override in config.\n` +
				`Available axes: ${Object.keys(font.variationAxes).join(', ')}\n` +
				`Add 'weight: <number>' to font config.`
		);
	} else {
		weight = font['OS/2']?.usWeightClass || inferWeight(font.subfamilyName);
	}

	// Log extracted family name for debugging
	logInfo(`  Font family: "${family}" (from ${familySource})`);

	return {
		family,
		weight,
		style,
		postscriptName: font.postscriptName,
		isVariable
	};
}

// ============================================================================
// Whitelist Validation
// ============================================================================

async function validateWhitelist(
	fontBuffer: Buffer,
	whitelist: string,
	sourcePath: string
): Promise<string[]> {
	const warnings: string[] = [];
	const fontOrCollection = fontkit.create(fontBuffer);
	if (!isFont(fontOrCollection)) {
		throw new Error(`Font collections are not supported: ${sourcePath}`);
	}
	const font = fontOrCollection;

	const uniqueChars = [...new Set(whitelist)];
	const missingChars: string[] = [];

	for (const char of uniqueChars) {
		const codePoint = char.codePointAt(0);
		if (!codePoint) continue;

		const glyph = font.glyphForCodePoint(codePoint);
		if (!glyph || glyph.id === 0) {
			missingChars.push(char);
		}
	}

	if (missingChars.length > 0) {
		warnings.push(
			`Font ${path.basename(sourcePath)} is missing ${missingChars.length} whitelisted characters: ` +
				`"${missingChars.join('')}"`
		);
	}

	return warnings;
}

// ============================================================================
// Font Subsetting
// ============================================================================

async function subsetFontFile(
	sourcePath: string,
	whitelist: string,
	outputPath: string,
	metadata: FontMetadata
): Promise<void> {
	const fontBuffer = await readFile(sourcePath);

	const warnings = await validateWhitelist(fontBuffer, whitelist, sourcePath);
	for (const warning of warnings) {
		logWarning(warning);
	}

	const normalizedWhitelist = [...new Set(whitelist.normalize('NFC'))].join('');

	const subsetBuffer = await subsetFont(fontBuffer, normalizedWhitelist, {
		targetFormat: 'woff2',
		...(metadata.isVariable && metadata.weight
			? {
					variationAxes: {
						wght: metadata.weight
					}
				}
			: {})
	});

	await mkdir(path.dirname(outputPath), { recursive: true });
	await writeFile(outputPath, subsetBuffer);
}

// ============================================================================
// Unicode Range Generation
// ============================================================================

function formatRange(start: number, end: number): string {
	const startHex = start.toString(16).toUpperCase();
	const endHex = end.toString(16).toUpperCase();

	if (start === end) {
		return `U+${startHex}`;
	}
	return `U+${startHex}-${endHex}`;
}

function generateRangeComment(whitelist: string, codePoints: number[]): string {
	const categories: string[] = [];

	const hasLowercase = codePoints.some((cp) => cp >= 0x61 && cp <= 0x7a);
	const hasUppercase = codePoints.some((cp) => cp >= 0x41 && cp <= 0x5a);
	const hasDigits = codePoints.some((cp) => cp >= 0x30 && cp <= 0x39);
	const hasPunctuation = codePoints.some(
		(cp) =>
			(cp >= 0x20 && cp <= 0x2f) ||
			(cp >= 0x3a && cp <= 0x40) ||
			(cp >= 0x5b && cp <= 0x60) ||
			(cp >= 0x7b && cp <= 0x7e)
	);

	if (hasUppercase && hasLowercase) {
		categories.push('letters');
	} else if (hasUppercase) {
		categories.push('uppercase');
	} else if (hasLowercase) {
		categories.push('lowercase');
	}

	if (hasDigits) categories.push('numbers');
	if (hasPunctuation) categories.push('punctuation');

	if (whitelist.length <= 20) {
		return `Only contains: ${whitelist}`;
	}

	return categories.length > 0 ? `${categories.join(', ')}` : `${codePoints.length} characters`;
}

function generateUnicodeRange(whitelist: string): UnicodeRange {
	const codePoints = [...new Set(whitelist)]
		.map((char) => char.codePointAt(0))
		.filter((cp): cp is number => cp !== undefined)
		.sort((a, b) => a - b);

	const ranges: string[] = [];
	let rangeStart = codePoints[0];
	let rangeEnd = codePoints[0];

	for (let i = 1; i < codePoints.length; i++) {
		const current = codePoints[i];

		if (current === rangeEnd + 1) {
			rangeEnd = current;
		} else {
			ranges.push(formatRange(rangeStart, rangeEnd));
			rangeStart = current;
			rangeEnd = current;
		}
	}

	ranges.push(formatRange(rangeStart, rangeEnd));

	return {
		cssRange: ranges.join(', '),
		comment: generateRangeComment(whitelist, codePoints)
	};
}

// ============================================================================
// CSS Generation
// ============================================================================

async function generateCssFile(fonts: FontFaceDescriptor[], cssOutputPath: string): Promise<void> {
	const lines = [
		'/* Auto-generated by vite-plugin-font-subset */',
		'/* Do not edit manually - changes will be overwritten */',
		'',
		'/* Subsetted fonts for optimal loading */',
		''
	];

	for (const font of fonts) {
		lines.push(
			`/* ${font.family} ${font.weight} - ${font.unicodeRange.comment} */`,
			'@font-face {',
			`\tfont-family: '${font.family}';`,
			`\tfont-weight: ${font.weight};`,
			`\tfont-style: ${font.style};`,
			`\tfont-display: swap;`,
			`\tsrc: url('/fonts/${path.basename(font.fontPath)}') format('woff2');`,
			`\tunicode-range: ${font.unicodeRange.cssRange};`,
			'}',
			''
		);
	}

	await writeFile(cssOutputPath, lines.join('\n'), 'utf-8');
}

// ============================================================================
// Cache Management
// ============================================================================

async function generateCacheKey(sourcePath: string, whitelist: string): Promise<string> {
	const sourceContent = await readFile(sourcePath);
	const hash = createHash('sha256');

	hash.update(sourceContent);
	hash.update(whitelist);

	return hash.digest('hex').substring(0, 16);
}

async function loadCacheManifest(cacheDir: string): Promise<Map<string, CacheEntry>> {
	const manifestPath = path.join(cacheDir, 'manifest.json');

	if (!existsSync(manifestPath)) {
		return new Map();
	}

	try {
		const content = await readFile(manifestPath, 'utf-8');
		const data = JSON.parse(content);
		return new Map(Object.entries(data));
	} catch {
		return new Map();
	}
}

async function saveCacheManifest(
	cacheDir: string,
	manifest: Map<string, CacheEntry>
): Promise<void> {
	const manifestPath = path.join(cacheDir, 'manifest.json');
	await mkdir(cacheDir, { recursive: true });

	const data = Object.fromEntries(manifest);
	await writeFile(manifestPath, JSON.stringify(data, null, 2), 'utf-8');
}

async function isCacheValid(
	entry: CacheEntry,
	sourcePath: string,
	whitelist: string
): Promise<boolean> {
	if (!existsSync(entry.outputPath)) {
		return false;
	}

	const currentHash = await generateCacheKey(sourcePath, whitelist);
	return entry.sourceHash === currentHash;
}

// ============================================================================
// Output Filename Generation
// ============================================================================

function generateOutputFilename(metadata: FontMetadata, sourcePath: string): string {
	const baseName = path.basename(sourcePath, path.extname(sourcePath));

	if (baseName.includes('-subset')) {
		return `${baseName}.woff2`;
	}

	const familySlug = metadata.family.toLowerCase().replace(/\s+/g, '-');
	return `${familySlug}-${metadata.weight}-${metadata.style}-subset.woff2`;
}

// ============================================================================
// Main Processing
// ============================================================================

async function processFonts(
	config: FontSubsetConfig,
	viteConfig: ResolvedConfig,
	isProduction: boolean
): Promise<void> {
	validateConfig(config);

	try {
		await checkDependencies();
	} catch (error) {
		if (!isProduction && config.skipOnMissingDeps !== false) {
			logWarning((error as Error).message);
			logInfo('Skipping font subsetting in development mode');
			return;
		}
		throw error;
	}

	const resolver = new PathResolver(viteConfig);
	const outputDir = resolver.resolveOutputDir(config.outputDir || 'static/fonts');
	const cssOutputPath = resolver.resolveCssPath(config.cssOutputPath || 'src/lib/fonts.css');
	const cacheDir = resolver.resolveCacheDir(
		config.cacheDir || 'node_modules/.vite-plugin-font-subset'
	);

	const cacheManifest = await loadCacheManifest(cacheDir);

	const fontDescriptors: FontFaceDescriptor[] = [];
	let subsettedCount = 0;
	let cachedCount = 0;

	for (const fontConfig of config.fonts) {
		const sourcePath = resolver.resolveSource(fontConfig.source);

		if (!existsSync(sourcePath)) {
			throw new Error(`Source font not found: ${sourcePath}`);
		}

		const cacheKey = await generateCacheKey(sourcePath, fontConfig.whitelist);
		const cacheEntry = cacheManifest.get(cacheKey);

		const metadata = await extractFontMetadata(sourcePath, {
			family: fontConfig.family,
			weight: fontConfig.weight,
			style: fontConfig.style
		});

		const outputFilename = generateOutputFilename(metadata, sourcePath);
		const outputPath = path.join(outputDir, outputFilename);

		if (cacheEntry && (await isCacheValid(cacheEntry, sourcePath, fontConfig.whitelist))) {
			logInfo(`Using cached subset: ${outputFilename}`);
			await copyFile(cacheEntry.outputPath, outputPath);
			cachedCount++;

			fontDescriptors.push({
				family: metadata.family,
				weight: metadata.weight,
				style: metadata.style,
				fontPath: outputPath,
				unicodeRange: cacheEntry.unicodeRange,
				originalSource: fontConfig.source
			});
		} else {
			logInfo(`Subsetting font: ${path.basename(sourcePath)} -> ${outputFilename}`);

			await subsetFontFile(sourcePath, fontConfig.whitelist, outputPath, metadata);
			subsettedCount++;

			const unicodeRange = generateUnicodeRange(fontConfig.whitelist);

			const cachedPath = path.join(cacheDir, `${cacheKey}-${outputFilename}`);
			await mkdir(cacheDir, { recursive: true });
			await copyFile(outputPath, cachedPath);

			cacheManifest.set(cacheKey, {
				sourceHash: cacheKey,
				outputPath: cachedPath,
				metadata,
				unicodeRange,
				timestamp: Date.now()
			});

			fontDescriptors.push({
				family: metadata.family,
				weight: metadata.weight,
				style: metadata.style,
				fontPath: outputPath,
				unicodeRange,
				originalSource: fontConfig.source
			});
		}
	}

	await saveCacheManifest(cacheDir, cacheManifest);
	await generateCssFile(fontDescriptors, cssOutputPath);

	logInfo(
		`Processed ${config.fonts.length} fonts (${subsettedCount} subsetted, ${cachedCount} cached)`
	);
	logInfo(`Generated: ${cssOutputPath}`);
}

// ============================================================================
// Plugin Export
// ============================================================================

export function fontSubsetPlugin(config: FontSubsetConfig): Plugin {
	let viteConfig: ResolvedConfig;
	let isProduction: boolean;

	return {
		name: 'vite-plugin-font-subset',

		configResolved(resolvedConfig) {
			viteConfig = resolvedConfig;
			isProduction = resolvedConfig.mode === 'production';
		},

		async buildStart() {
			try {
				await processFonts(config, viteConfig, isProduction);
			} catch (error) {
				if (isProduction) {
					this.error(`Font subsetting failed: ${(error as Error).message}`);
				} else if (!config.skipOnMissingDeps) {
					this.error(`Font subsetting failed: ${(error as Error).message}`);
				} else {
					logWarning(`Font subsetting skipped: ${(error as Error).message}`);
				}
			}
		}
	};
}
