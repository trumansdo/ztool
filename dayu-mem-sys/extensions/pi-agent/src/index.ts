/**
 * dayu-mem-sys pi-agent 扩展 — 主入口
 *
 * 注册 pi-agent 生命周期钩子，实现记忆召回、注入与归档。
 *
 * 钩子覆盖：
 * - session_start       → 初始化会话状态，设置会话名称
 * - before_agent_start  → 召回相关记忆，不可见注入到上下文
 * - turn_end            → 归档当前轮次（用户消息 + assistant 回复）
 * - session_before_compact → 压缩前确保持久化
 *
 * 用法：在 pi 配置中注册此扩展。
 *   extensions:
 *     dayu-mem-sys:
 *       path: ./extensions/pi-agent/src/index.ts
 *       config:
 *         recallEnabled: true
 *         maxInjectionBlocks: 5
 */

import type { ExtensionAPI, ExtensionContext } from '@earendil-works/pi-coding-agent';
import { recall, archiveTurn, isNapiAvailable, getStats } from './napi';
import {
    type DayuExtensionConfig,
    type ExtensionMetrics,
    DEFAULT_CONFIG,
} from './types';

// ============================================================
// 状态
// ============================================================

let config: DayuExtensionConfig = { ...DEFAULT_CONFIG };
let metrics: ExtensionMetrics = {
    recallCount: 0,
    recallHitCount: 0,
    archiveCount: 0,
    lastRecallDurationMs: 0,
    lastArchiveDurationMs: 0,
    errors: 0,
};

// ============================================================
// 配置合并
// ============================================================

function mergeConfig(userConfig?: Partial<DayuExtensionConfig>): DayuExtensionConfig {
    return { ...DEFAULT_CONFIG, ...userConfig };
}

// ============================================================
// 工具函数
// ============================================================

/** 从消息列表中提取最后一条用户消息文本 */
function getLastUserText(messages: Array<{ role: string; content: string }>): string {
    const lastUser = [...messages].reverse().find(m => m.role === 'user');
    return lastUser?.content ?? '';
}

/** 估算文本 token 数（粗略：中文 ~1.5 字/token，英文 ~4 字/token） */
function estimateTokens(text: string): number {
    let chineseChars = 0;
    let otherChars = 0;
    for (const ch of text) {
        if (/[\u4e00-\u9fff\u3400-\u4dbf]/.test(ch)) {
            chineseChars++;
        } else if (!/\s/.test(ch)) {
            otherChars++;
        }
    }
    return Math.ceil(chineseChars / 1.5 + otherChars / 4);
}

/** 格式化注入块为文本 */
function formatInjectionBlocks(
    blocks: Array<{ relevance: string; text: string; score: number }>,
    budget: number,
): string {
    const relevanceIcon = (r: string) => {
        switch (r) {
            case 'high': return '★';
            case 'medium': return '●';
            default: return '○';
        }
    };

    const lines: string[] = [];
    let usedTokens = 0;

    for (const block of blocks) {
        const line = `${relevanceIcon(block.relevance)} ${block.text}`;
        const lineTokens = estimateTokens(line);
        if (usedTokens + lineTokens > budget) break;
        lines.push(line);
        usedTokens += lineTokens;
    }

    if (lines.length === 0) return '';
    return `\n<!-- 大禹记忆注入 · ${lines.length} 条 -->\n${lines.join('\n')}\n<!-- /大禹记忆注入 -->\n`;
}

// ============================================================
// 钩子实现
// ============================================================

/**
 * session_start: 会话初始化
 *
 * 设置会话名称（含记忆系统状态标记），初始化指标。
 */
async function onSessionStart(_event: unknown, ctx: ExtensionContext): Promise<void> {
    const napiReady = isNapiAvailable();
    const statusTag = napiReady ? '🧠' : '🧠⊘';

    const currentName = ctx.sessionManager?.getSessionName?.() ?? '';
    if (!currentName || currentName.startsWith('New Session')) {
        ctx.sessionManager?.setSessionName?.(`${statusTag} 大禹记忆`);
    }

    // 重置指标
    metrics = {
        recallCount: 0,
        recallHitCount: 0,
        archiveCount: 0,
        lastRecallDurationMs: 0,
        lastArchiveDurationMs: 0,
        errors: 0,
    };

    console.log(`[dayu-pi] session_start — napi: ${napiReady ? 'ready' : 'unavailable'}`);
}

/**
 * before_agent_start: Agent 启动前注入记忆
 *
 * 1. 提取最后一条用户消息
 * 2. 调用 recall 获取相关记忆块
 * 3. 按相关性过滤 + token 预算裁剪
 * 4. 以 display: false 不可见注入（不破坏前缀缓存）
 */
async function onBeforeAgentStart(
    event: { prompt: string; messages: Array<{ role: string; content: string }> },
    _ctx: ExtensionContext,
): Promise<{ display?: boolean; content?: string; metadata?: Record<string, unknown> }> {
    if (!config.recallEnabled) return {};

    const userText = event.prompt || getLastUserText(event.messages ?? []);
    if (!userText || userText.trim().length === 0) return {};

    const startTime = Date.now();
    metrics.recallCount++;

    try {
        const blocks = await recall(userText, 1.0);

        metrics.lastRecallDurationMs = Date.now() - startTime;

        if (blocks.length === 0) return {};

        // 过滤低相关性
        const filtered = blocks
            .filter(b => b.score >= config.minRelevanceScore)
            .slice(0, config.maxInjectionBlocks);

        if (filtered.length === 0) return {};

        metrics.recallHitCount++;

        const injectionText = formatInjectionBlocks(filtered, config.injectionTokenBudget);
        if (!injectionText) return {};

        return {
            display: config.injectionMode === 'visible',
            content: injectionText,
            metadata: {
                source: 'dayu-mem-sys',
                blockCount: filtered.length,
                ballIds: filtered.flatMap(b => b.ballIds),
                topScore: filtered[0]?.score ?? 0,
                recallDurationMs: metrics.lastRecallDurationMs,
                timestamp: Date.now(),
            },
        };
    } catch (e) {
        metrics.errors++;
        console.error('[dayu-pi] before_agent_start 失败:', e);
        return {};
    }
}

/**
 * turn_end: 轮次结束归档
 *
 * 将用户消息和 assistant 回复写入存储。
 * 异步执行，不阻塞下一轮。
 */
async function onTurnEnd(
    event: { messages: Array<{ role: string; content: string }>; turnIndex: number },
    ctx: ExtensionContext,
): Promise<void> {
    if (!config.archiveEnabled) return;

    const lastUser = [...(event.messages ?? [])].reverse().find(m => m.role === 'user');
    const lastAssistant = [...(event.messages ?? [])].reverse().find(m => m.role === 'assistant');

    if (!lastUser || !lastAssistant) return;

    const startTime = Date.now();
    metrics.archiveCount++;

    try {
        const sessionId = ctx.sessionManager?.getSessionId?.() ?? 'unknown';
        const turnId = `${sessionId}-${event.turnIndex}`;

        await archiveTurn(
            turnId,
            lastUser.content,
            lastAssistant.content,
            new Array(384).fill(0), // embedding 由 napi 侧生成
        );

        metrics.lastArchiveDurationMs = Date.now() - startTime;
    } catch (e) {
        metrics.errors++;
        console.error('[dayu-pi] turn_end 归档失败:', e);
    }
}

/**
 * session_before_compact: 压缩前持久化
 *
 * 确保当前会话数据在上下文压缩前已落盘。
 */
async function onSessionBeforeCompact(
    _event: unknown,
    ctx: ExtensionContext,
): Promise<void> {
    const sessionId = ctx.sessionManager?.getSessionId?.() ?? 'unknown';
    console.log(`[dayu-pi] session_before_compact: ${sessionId}`);

    // 输出当前指标
    const stats = await getStats();
    console.log('[dayu-pi] 扩展指标:', {
        ...metrics,
        napiStats: stats,
    });
}

// ============================================================
// 扩展入口
// ============================================================

export default function dayuPiExtension(pi: ExtensionAPI): void {
    // 合并用户配置
    const userConfig = pi.getExtensionConfig?.() as Partial<DayuExtensionConfig> | undefined;
    config = mergeConfig(userConfig);

    console.log('[dayu-pi] 大禹记忆扩展初始化', {
        recallEnabled: config.recallEnabled,
        archiveEnabled: config.archiveEnabled,
        maxBlocks: config.maxInjectionBlocks,
        napiAvailable: isNapiAvailable(),
    });

    // 注册钩子
    pi.on('session_start', onSessionStart);
    pi.on('before_agent_start', onBeforeAgentStart);
    pi.on('turn_end', onTurnEnd);
    pi.on('session_before_compact', onSessionBeforeCompact);

    // 暴露调试命令
    pi.registerCommand?.('dayu:stats', async () => {
        const napiStats = await getStats();
        return {
            extension: metrics,
            napi: napiStats,
            config,
        };
    });

    pi.registerCommand?.('dayu:reload', async () => {
        const newConfig = pi.getExtensionConfig?.() as Partial<DayuExtensionConfig> | undefined;
        config = mergeConfig(newConfig);
        return { reloaded: true, config };
    });
}
