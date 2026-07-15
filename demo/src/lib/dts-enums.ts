import { readFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import ts from 'typescript'

const require = createRequire(import.meta.url)
const dtsPath = require.resolve('img2lv/index.d.ts')
const source = readFileSync(dtsPath, 'utf8')
const sourceFile = ts.createSourceFile(dtsPath, source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS)

function getStringEnumValues(enumName: string): string[] {
  for (const statement of sourceFile.statements) {
    if (!ts.isEnumDeclaration(statement) || statement.name.text !== enumName) continue

    return statement.members.map((member) => {
      if (member.initializer && ts.isStringLiteral(member.initializer)) return member.initializer.text
      return member.name.getText(sourceFile)
    })
  }

  throw new Error(`Could not find enum ${enumName} in img2lv/index.d.ts`)
}

export const COLOR_FORMATS = getStringEnumValues('ColorFormat')
export const COMPRESS_METHODS = getStringEnumValues('CompressMethod')
