import { invoke } from "@tauri-apps/api/core";

export async function invokeRead(args: ReadCommandOptions) {
    await invoke("read", args);
}

export async function invokeCompile(args: CompileCommandOptions): Promise<string> {
    return await invoke<string>("compile", args);
}

export async function invokeEscapeText(args: { text: string }): Promise<string> {
    return await invoke<string>("escape_text", args);
}

export async function invokeReadLastLine(args: { filePath: string }): Promise<string> {
    return await invoke<string>("read_last_line", args);
}
