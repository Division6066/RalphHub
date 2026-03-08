import { browser } from '$app/environment';

export function isDesktopRuntime(): boolean {
	return browser && typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export async function invokeTauri<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isDesktopRuntime()) {
		throw new Error('Tauri runtime is not available in the current browser session.');
	}

	const { invoke } = await import('@tauri-apps/api/core');
	return invoke<T>(command, args);
}
