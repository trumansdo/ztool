// 统计原语：所有脚本共用，避免同一个 cos / 分位数 / 符号检验写出多个版本

const crypto = require('crypto');

/** 余弦（向量已归一化时等于点积，但这里不假设归一化） */
function cos(a, b) {
  let s = 0;
  for (let i = 0; i < a.length; i++) s += a[i] * b[i];
  return s;
}

const mean = (arr) => (arr.length ? arr.reduce((x, y) => x + y, 0) / arr.length : 0);

const sum = (arr) => arr.reduce((x, y) => x + y, 0);

/** 线性插值分位数，p ∈ [0,100] */
function percentile(arr, p) {
  if (!arr.length) return null;
  const s = [...arr].sort((x, y) => x - y);
  const idx = Math.min(s.length - 1, Math.max(0, Math.round((p / 100) * (s.length - 1))));
  return s[idx];
}

const median = (arr) => percentile(arr, 50);

/** 降序下标（按 score[i]） */
function orderDesc(scores) {
  return scores.map((_, i) => i).sort((x, y) => scores[y] - scores[x]);
}

const hashOf = (text) => crypto.createHash('sha1').update(text).digest('hex');

const erf = (x) => {
  const s = Math.sign(x);
  x = Math.abs(x);
  const a1 = 0.254829592;
  const a2 = -0.284496736;
  const a3 = 1.421413741;
  const a4 = -1.453152027;
  const a5 = 1.061405429;
  const p = 0.3275911;
  const t = 1 / (1 + p * x);
  return s * (1 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * Math.exp(-x * x));
};

/**
 * 符号检验（双侧，正态近似 + 连续性校正）。
 * 用于「同一批查询上逐条对比两个配置/模型」，比比均值更有说服力。
 */
function signTest(win, lose) {
  const n = win + lose;
  if (!n) return 1;
  const z = (Math.abs(win - lose) - 1) / Math.sqrt(n);
  return Math.max(0, Math.min(1, 2 * (1 - 0.5 * (1 + erf(z / Math.SQRT2)))));
}

/** 逐条配对对比，返回 {win, tie, lose, p, meanA, meanB} */
function paired(a, b, pick) {
  let win = 0;
  let tie = 0;
  let lose = 0;
  let sumA = 0;
  let sumB = 0;
  for (let i = 0; i < a.length; i++) {
    const x = pick(a[i]);
    const y = pick(b[i]);
    sumA += x;
    sumB += y;
    if (Math.abs(x - y) < 1e-9) tie++;
    else if (x > y) win++;
    else lose++;
  }
  return {
    win,
    tie,
    lose,
    p: signTest(win, lose),
    meanA: a.length ? sumA / a.length : 0,
    meanB: b.length ? sumB / b.length : 0,
  };
}

const pctStr = (x) => (x === null || x === undefined ? 'n/a' : `${(x * 100).toFixed(1)}%`);
const f3 = (x) => (x === null || x === undefined ? 'n/a' : x.toFixed(3));
const pad = (v, n) => String(v).padStart(n);

module.exports = { cos, mean, sum, percentile, median, orderDesc, hashOf, erf, signTest, paired, pctStr, f3, pad };
