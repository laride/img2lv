#!/usr/bin/env node

/**
 * Generate API documentation from img2lv's index.d.ts using TypeDoc.
 * Outputs a single markdown file consumed by the Astro /doc page at build time.
 */

import { execSync } from 'node:child_process'
import { mkdirSync, existsSync } from 'node:fs'
import { resolve } from 'node:path'

const root = resolve(import.meta.dirname, '..')
const outDir = resolve(root, 'src/generated')

if (!existsSync(outDir)) {
  mkdirSync(outDir, { recursive: true })
}

const dtsPath = resolve(root, 'node_modules/img2lv/index.d.ts')
if (!existsSync(dtsPath)) {
  console.error('✗ img2lv/index.d.ts not found. Run pnpm install first.')
  process.exit(1)
}

execSync(
  [
    'npx typedoc',
    `--entryPoints "${dtsPath}"`,
    '--plugin typedoc-plugin-markdown',
    `--out "${outDir}"`,
    '--tsconfig tsconfig.typedoc.json',
    '--readme none',
    '--hidePageHeader',
    '--hideGroupHeadings',
    '--hideBreadcrumbs',
    '--disableSources',
    '--outputFileStrategy modules',
    '--flattenOutputFiles',
  ].join(' '),
  { cwd: root, stdio: 'inherit' },
)

console.log(`\n✓ API docs generated in ${outDir}`)
