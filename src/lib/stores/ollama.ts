import { writable, derived } from 'svelte/store';

export type OllamaState = {
	installed: boolean;
	running: boolean;
	endpoint: string;
	defaultModel: string;
	availableModels: string[];
	localFirst: boolean;
};

const defaultOllamaState: OllamaState = {
	installed: false,
	running: false,
	endpoint: 'http://localhost:11434',
	defaultModel: 'mistral',
	availableModels: [],
	localFirst: true
};

export const ollamaStore = writable<OllamaState>(defaultOllamaState);

export const ollamaReady = derived(
	ollamaStore,
	($o) => $o.installed && $o.running
);

export const effectiveModel = derived(
	ollamaStore,
	($o) => ($o.localFirst && $o.running ? `ollama/${$o.defaultModel}` : null)
);

export function setOllamaStatus(status: Partial<OllamaState>) {
	ollamaStore.update((s) => ({ ...s, ...status }));
}
