// 配置分层：base.toml 写全量参数，各后端配置只写差异，运行时覆盖最后应用
//
//   config/base.toml        —— 唯一完整配置（所有字段都有值）
//   config/onnx.toml        —— 本地 ONNX 模型（只含 embedding_* 差异）
//   config/http.toml        —— 远端 HTTP 嵌入（含密钥，不入库）
//   config/http.example.toml—— 同上，密钥替换为占位符（入库）
//   config/lms.toml         —— 本机 LM Studio 0.6B
//
// 合并按行进行：只覆盖 `key = value` 形式的行，注释与空行保留 base 的。
// 这比引入 TOML 解析器简单，也够用——测试配置是扁平结构（Config 无嵌套表）。

const fs = require('fs');
const path = require('path');
const { CONFIG_DIR } = require('./paths.cjs');

const KEY_LINE = /^(\s*)([A-Za-z_][A-Za-z0-9_]*)(\s*=\s*)(.*)$/;

/** 把文本拆成 [{line, key, indent, value}]，非 key 行为 null key */
function parseLines(text) {
  return text.split(/\r?\n/).map((line) => {
    const m = line.match(KEY_LINE);
    return m ? { line, indent: m[1], key: m[2], sep: m[3], value: m[4] } : { line, key: null };
  });
}

function formatValue(v) {
  if (typeof v === 'boolean') return String(v);
  if (typeof v === 'number') return Number.isInteger(v) ? String(v) : String(v);
  if (typeof v === 'string') return `"${v}"`;
  throw new Error(`配置值类型不支持：${typeof v}`);
}

/**
 * 用 overrides 覆盖 baseText 中的同名键。
 * 已存在的键就地替换（保留缩进），不存在的键追加到末尾。
 */
function applyOverrides(baseText, overrides) {
  const pending = new Map(Object.entries(overrides).map(([k, v]) => [k, formatValue(v)]));
  const out = parseLines(baseText).map((p) => {
    if (!p.key || !pending.has(p.key)) return p.line;
    const v = pending.get(p.key);
    pending.delete(p.key);
    return `${p.indent}${p.key}${p.sep}${v}`;
  });
  if (pending.size) {
    out.push('', '# --- 运行时覆盖（base.toml 未声明的键）---');
    for (const [k, v] of pending) out.push(`${k} = ${v}`);
  }
  return out.join('\n');
}

function readConfigFile(name) {
  const p = path.join(CONFIG_DIR, `${name}.toml`);
  if (!fs.existsSync(p)) {
    throw new Error(`配置不存在：${p}\n可用配置：${listConfigs().join(', ')}`);
  }
  return fs.readFileSync(p, 'utf8');
}

function listConfigs() {
  if (!fs.existsSync(CONFIG_DIR)) return [];
  return fs
    .readdirSync(CONFIG_DIR)
    .filter((f) => f.endsWith('.toml') && !f.endsWith('.example.toml') && f !== 'base.toml')
    .map((f) => path.basename(f, '.toml'))
    .sort();
}

/**
 * 加载配置：base.toml ← config/<name>.toml ← overrides
 *
 * @param {string} name      后端配置名（onnx / http / lms）
 * @param {object} overrides 运行时覆盖（db_path / log_dir / 参数）
 * @returns {string} 可直接交给 api.init 的 TOML 文本
 */
function load(name, overrides = {}) {
  const base = readConfigFile('base');
  const overlay = name === 'base' ? '' : readConfigFile(name);
  const merged = applyOverrides(base, keyValues(overlay));
  return applyOverrides(merged, overrides);
}

/** 提取一个配置文件里的 key = value（用于分层覆盖） */
function keyValues(text) {
  const obj = {};
  for (const p of parseLines(text)) {
    if (!p.key) continue;
    obj[p.key] = parseRaw(p.value);
  }
  return obj;
}

function parseRaw(raw) {
  const v = raw.replace(/\s+#.*$/, '').trim();
  if (v === 'true') return true;
  if (v === 'false') return false;
  if (/^-?\d+$/.test(v)) return parseInt(v, 10);
  if (/^-?\d*\.\d+$/.test(v)) return parseFloat(v);
  const s = v.match(/^"(.*)"$/);
  return s ? s[1] : v;
}

module.exports = { load, keyValues, applyOverrides, listConfigs, parseRaw };
