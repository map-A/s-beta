import { readdirSync, readFileSync, writeFileSync, existsSync } from 'fs'
import { join, resolve } from 'path'
import { fileURLToPath } from 'url'

const __dirname = fileURLToPath(new URL('.', import.meta.url))
const DATA_DIR = resolve(__dirname, '../../outputs')
const OUTPUT_FILE = join(DATA_DIR, 'companies.json')

// Match folder names like: 移远通信_603236, 天齐锂业_002466
const FOLDER_PATTERN = /^(.+)_(\d{6})$/

function generate() {
  if (!existsSync(DATA_DIR)) {
    console.error(`Data directory not found: ${DATA_DIR}`)
    process.exit(1)
  }

  const entries = readdirSync(DATA_DIR, { withFileTypes: true })
  const companies = []

  for (const entry of entries) {
    if (!entry.isDirectory()) continue

    const match = entry.name.match(FOLDER_PATTERN)
    if (!match) continue

    const [, name, code] = match
    const metaPath = join(DATA_DIR, entry.name, 'meta.json')

    if (!existsSync(metaPath)) {
      console.warn(`  ⚠ ${entry.name}/meta.json 不存在，跳过`)
      continue
    }

    try {
      const meta = JSON.parse(readFileSync(metaPath, 'utf-8'))
      companies.push(meta)
      console.log(`  ✓ ${name} (${code})`)
    } catch (err) {
      console.error(`  ✗ ${entry.name}/meta.json 解析失败:`, err.message)
    }
  }

  // Sort by code for stable output
  companies.sort((a, b) => a.code.localeCompare(b.code))

  const output = JSON.stringify(companies, null, 2)
  writeFileSync(OUTPUT_FILE, output, 'utf-8')

  console.log(`\n✅ 已生成 companies.json（${companies.length} 家公司）→ ${OUTPUT_FILE}`)
}

generate()
