/**
 * nianxiang-dsh-plugin: 大禹记忆网络 dsh 适配层
 *
 * 注册 dsh 钩子：
 * - session/event: 触发 archive_turn，归档用户消息
 * - agent/pre-step: 触发 recall，尾部注入记忆块
 *
 * 用法：在 dsh 配置中注册此插件。
 */

// ============================================================
// 类型定义
// ============================================================

interface DshContext {
    sessionId: string;
    turnIndex: number;
    messages: Array<{ role: string; content: string }>;
    systemPrompt?: string;
}

interface DshHookResult {
    appendToTail?: string;
    metadata?: Record<string, unknown>;
}

// ============================================================
// 钩子实现
// ============================================================

/**
 * session/event: 用户消息事件触发归档
 *
 * 将用户消息异步写入存储，不阻塞主流程。
 */
export async function onSessionEvent(context: DshContext): Promise<void> {
    const lastUser = context.messages
        .filter(m => m.role === 'user')
        .pop();

    if (!lastUser) {
        return;
    }

    try {
        await archiveTurnNative(
            `${context.sessionId}-${context.turnIndex}`,
            lastUser.content,
            new Array(384).fill(0), // embedding 由 napi 侧生成
        );
    } catch (e) {
        console.error('[dayu-dsh] archive failed:', e);
    }
}

/**
 * agent/pre-step: Agent 执行前注入记忆
 *
 * 调用 recall 获取相关记忆块，追加到消息尾部。
 * 不修改 system prompt，保证前缀缓存不被破坏。
 */
export async function onAgentPreStep(context: DshContext): Promise<DshHookResult> {
    const lastUser = context.messages
        .filter(m => m.role === 'user')
        .pop();

    if (!lastUser) {
        return {};
    }

    try {
        const blocks = await recallNative(lastUser.content, 1.0);

        if (blocks.length === 0) {
            return {};
        }

        // 拼装尾部注入文本
        const injectionText = blocks
            .map(b => {
                const relevanceTag = b.relevance === 'high' ? '★' :
                                     b.relevance === 'medium' ? '●' : '○';
                return `${relevanceTag} ${b.text}`;
            })
            .join('\n');

        return {
            appendToTail: `\n\n--- 相关记忆 ---\n${injectionText}\n---`,
            metadata: {
                source: 'dayu-mem-sys',
                ballIds: blocks.flatMap(b => b.ballIds),
                blockCount: blocks.length,
                timestamp: Date.now(),
            },
        };
    } catch (e) {
        console.error('[dayu-dsh] recall failed:', e);
        return {};
    }
}

// ============================================================
// Native 调用桩（实际由 napi 绑定提供）
// ============================================================

interface InjectionBlockDto {
    relevance: 'high' | 'medium' | 'low';
    text: string;
    ballIds: number[];
}

declare function recallNative(query: string, significance: number): Promise<InjectionBlockDto[]>;
declare function archiveTurnNative(
    turnId: string,
    userText: string,
    embedding: number[],
): Promise<void>;

// 兜底实现（napi 未加载时）
async function recallNative(_query: string, _sig: number): Promise<InjectionBlockDto[]> {
    return [];
}

async function archiveTurnNative(
    _turnId: string,
    _userText: string,
    _embedding: number[],
): Promise<void> {
    // noop
}

// ============================================================
// 导出
// ============================================================

export default {
    onSessionEvent,
    onAgentPreStep,
};
