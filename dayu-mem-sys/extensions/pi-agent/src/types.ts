/**
 * dayu-mem-sys pi-agent 扩展 — 类型定义
 */

// ============================================================
// 注入块
// ============================================================

export interface InjectionBlock {
    /** 相关性等级 */
    relevance: 'high' | 'medium' | 'low';
    /** 记忆文本 */
    text: string;
    /** 来源水球 ID 列表 */
    ballIds: number[];
    /** 相关性分数 (0-1) */
    score: number;
}

// ============================================================
// 扩展配置
// ============================================================

export interface DayuExtensionConfig {
    /** 是否启用记忆召回注入（默认 true） */
    recallEnabled: boolean;
    /** 是否启用轮次归档（默认 true） */
    archiveEnabled: boolean;
    /** 最大注入块数（默认 5） */
    maxInjectionBlocks: number;
    /** 注入模式：hidden(不可见) | visible(可见)（默认 hidden） */
    injectionMode: 'hidden' | 'visible';
    /** 最小相关性阈值，低于此值不注入（默认 0.3） */
    minRelevanceScore: number;
    /** 注入块 token 预算上限（默认 500） */
    injectionTokenBudget: number;
    /** napi 模块路径（默认自动检测） */
    napiModulePath?: string;
}

export const DEFAULT_CONFIG: DayuExtensionConfig = {
    recallEnabled: true,
    archiveEnabled: true,
    maxInjectionBlocks: 5,
    injectionMode: 'hidden',
    minRelevanceScore: 0.3,
    injectionTokenBudget: 500,
};

// ============================================================
// 归档条目
// ============================================================

export interface ArchiveEntry {
    turnId: string;
    sessionId: string;
    turnIndex: number;
    userText: string;
    assistantText: string;
    timestamp: number;
}

// ============================================================
// 指标
// ============================================================

export interface ExtensionMetrics {
    recallCount: number;
    recallHitCount: number;
    archiveCount: number;
    lastRecallDurationMs: number;
    lastArchiveDurationMs: number;
    errors: number;
}
