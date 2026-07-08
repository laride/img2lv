#!/usr/bin/env node

/**
 * Verify that the demo's img2lv dependency version and README content
 * are in sync with the main branch's root package.json and README.
 */

import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const mainBranch = process.argv[2] || 'origin/main'
const root = resolve(import.meta.dirname, '../..')

console.log(`Comparing against: ${mainBranch}\n`)

function git(args) {
  return execSync(`git ${args}`, { cwd: root, encoding: 'utf8' }).trim()
}

// --- Check img2lv version ---

const mainPkg = JSON.parse(git(`show ${mainBranch}:package.json`))
const demoPkg = JSON.parse(readFileSync(resolve(root, 'demo/package.json'), 'utf8'))
const rootVersion = mainPkg.version
const demoDep = demoPkg.dependencies['img2lv']
const demoVersion = demoDep.replace(/^[^\d]*/, '')

console.log(`Root package version (${mainBranch}): ${rootVersion}`)
console.log(`Demo img2lv dependency: ${demoDep} (base: ${demoVersion})`)

if (rootVersion !== demoVersion) {
  console.error(
    `\n✗ Demo img2lv version (${demoVersion}) does not match ${mainBranch} package version (${rootVersion}).`,
  )
  process.exit(1)
}
console.log('✓ img2lv version matches.\n')

// --- Check README ---

const mainReadme = git(`show ${mainBranch}:README.md`)
const demoReadmePath = resolve(root, 'demo/src/README.md')

const demoReadme = readFileSync(demoReadmePath, 'utf8').trim()

if (mainReadme !== demoReadme) {
  console.error(`✗ demo/src/README.md content differs from ${mainBranch} README.md.`)
  console.error(
    `\nTo sync from ${mainBranch}, run from the repository root:\n` +
      `  git show ${mainBranch}:README.md > demo/src/README.md`,
  )
  process.exit(1)
}
console.log('✓ README is in sync.\n')

console.log('All checks passed.')
