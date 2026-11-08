/**
 * dayu-mem-sys pi-agent 扩展 — napi 绑定加载层
 *
 * 负责加载 dayu-napi 原生模块，提供类型安全的调用接口。
 * 加载失败时自动降级为 noop 实现，确保扩展不会因缺失原生模块而崩溃。
 */

import type { InjectionBlock } from './types';

// ============================================================
// 原生模块接口
// ============================================================

interface DayuNapi {
    recall(query: string, significance: number): Promise<InjectionBlock[]>;
    archiveTurn(
        turnId: string,
        userText: string,
        assistantText: string,
        embedding: number[],
    ): Promise<void>;
    getStats(): Promise<Record<string, number>>;
}

// ============================================================
// 模块加载
// ============================================================

let napiModule: DayuNapi | null = null;
let loadAttempted = false;

function getNapiModule(): DayuNapi | null {
    if (loadAttempted) return napiModule;
    loadAttempted = true;

    try {
        // 尝试从工作区加载 dayu-napi
        // eslint-disable-next-line @typescript-eslint/no-require-imports
        const mod = require('@dayu-mem-sys/napi');
        napiModule = mod as DayuNapi;
        console.log('[dayu-pi] napi 模块加载成功');
    } catch {
        console.warn('[dayu-pi] napi 模块未找到，使用 noop 降级实现');
        napiModule = null;
    }

    return napiModule;
}

// ============================================================
// 公开 API（带降级）
// ============================================================

export async function recall(
    query: string,
    significance: number,
): Promise<InjectionBlock[]> {
    const mod = getNapiModule();
    if (!mod) return [];

    try {
        return await mod.recall(query, significance);
    } catch (e) {
        console.error('[dayu-pi] recall 调用失败:', e);
        return [];
    }
}

export async function archiveTurn(
    turnId: string,
    userText: string,
    assistantText: string,
    embedding: number[],
): Promise<void> {
    const mod = getNapiModule();
    if (!mod) return;

    try {
        await mod.archiveTurn(turnId, userText, assistantText, embedding);
    } catch (e) {
        console.error('[dayu-pi] archiveTurn 调用失败:', e);
    }
}

export async function getStats(): Promise<Record<string, number>> {
    const mod = getNapiModule();
    if (!mod) return {};

    try {
        return await mod.getStats();
    } catch {
        return {};
    }
}

/** 检查原生模块是否可用 */
export function isNapiAvailable(): boolean {
    return getNapiModule() !== null;
}
