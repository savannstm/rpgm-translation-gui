import { invoke } from "@tauri-apps/api/core";

export async function invokeRead(args: ReadCommandOptions) {
    await invoke("read", args);
}

export async function invokeCompile(args: CompileCommandOptions): Promise<string> {
    return await invoke<string>("compile", args);
}
