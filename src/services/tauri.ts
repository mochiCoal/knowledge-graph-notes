import { invoke } from '@tauri-apps/api/core'

/**
 * Tauri 命令调用失败时抛出的统一错误类型。
 *
 * Rust 端命令统一返回 `Result<T, String>`，失败时 invoke 会直接抛出该字符串；
 * 这里包装成 Error，方便界面提示、日志记录以及后续统一错误处理。
 */
export class CommandError extends Error {
  constructor(
    readonly command: string,
    message: string,
  ) {
    super(message)
    this.name = 'CommandError'
  }
}

function toMessage(error: unknown): string {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message

  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

/**
 * 调用 Rust 命令并统一错误处理。
 *
 * 业务模块（阶段二起）统一通过该函数访问后端，不直接使用 invoke，
 * 便于后续加入重试、日志、加载状态等通用逻辑。
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw new CommandError(command, toMessage(error))
  }
}
