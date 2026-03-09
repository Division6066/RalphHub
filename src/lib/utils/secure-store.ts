import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

// Legacy fixed key fields (kept for backwards compatibility)
export const KEY_FIELDS = [
	'ANTHROPIC_API_KEY',
	'OPENAI_API_KEY',
	'GROK_API_KEY',
	'GEMINI_API_KEY',
	'PERPLEXICA_KEYS'
] as const;

export type KeyField = (typeof KEY_FIELDS)[number];
export type KeyMap = Record<string, string>;

type SecureStoreConfig = {
	vaultPath: string;
	clientName: string;
	vaultPassword: string;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function createEmptyKeyMap(): Record<KeyField, string> {
	return {
		ANTHROPIC_API_KEY: '',
		OPENAI_API_KEY: '',
		GROK_API_KEY: '',
		GEMINI_API_KEY: '',
		PERPLEXICA_KEYS: ''
	};
}

/**
 * Load all keys from Stronghold (desktop) or localStorage (browser).
 * Returns a Record<keyName, value>.
 */
export async function loadKeys(): Promise<KeyMap> {
	if (!isDesktopRuntime()) {
		// Browser fallback: load from localStorage
		const result: KeyMap = {};
		for (let i = 0; i < localStorage.length; i++) {
			const k = localStorage.key(i);
			if (k?.startsWith('amitos_key_')) {
				const keyName = localStorage.getItem(k) ?? '';
				// key is stored as amitos_key_{providerid} -> value
				result[k.replace('amitos_key_', '')] = localStorage.getItem(k) ?? '';
			}
		}
		return result;
	}

	const [{ Stronghold }, config] = await Promise.all([
		import('@tauri-apps/plugin-stronghold'),
		invokeTauri<SecureStoreConfig>('get_secure_store_config')
	]);
	const stronghold = await Stronghold.load(config.vaultPath, config.vaultPassword);
	const client = await getClient(stronghold, config.clientName);
	const store = client.getStore();

	const result: KeyMap = {};

	try {
		// Load legacy fixed keys
		for (const field of KEY_FIELDS) {
			try {
				const storedValue = await store.get(field);
				if (storedValue?.length) {
					result[field] = decodeValue(storedValue);
				}
			} catch {}
		}
		return result;
	} finally {
		await stronghold.unload();
	}
}

/**
 * Save a map of keyName -> value to Stronghold (desktop) or localStorage (browser).
 */
export async function saveKeys(values: KeyMap): Promise<void> {
	if (!isDesktopRuntime()) {
		for (const [key, value] of Object.entries(values)) {
			if (value?.trim()) {
				localStorage.setItem(`amitos_key_${key}`, value);
			} else {
				localStorage.removeItem(`amitos_key_${key}`);
			}
		}
		return;
	}

	const [{ Stronghold }, config] = await Promise.all([
		import('@tauri-apps/plugin-stronghold'),
		invokeTauri<SecureStoreConfig>('get_secure_store_config')
	]);
	const stronghold = await Stronghold.load(config.vaultPath, config.vaultPassword);
	const client = await getClient(stronghold, config.clientName);
	const store = client.getStore();

	try {
		for (const [field, value] of Object.entries(values)) {
			const trimmed = (value ?? '').trim();
			if (trimmed) {
				await store.insert(field, Array.from(encoder.encode(trimmed)));
			} else {
				try { await store.remove(field); } catch {}
			}
		}
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

async function getClient(
	stronghold: { loadClient(name: string): Promise<any>; createClient(name: string): Promise<any> },
	clientName: string
) {
	try {
		return await stronghold.loadClient(clientName);
	} catch {
		return await stronghold.createClient(clientName);
	}
}

function decodeValue(value: Uint8Array | number[] | null): string {
	if (!value?.length) return '';
	return decoder.decode(new Uint8Array(value));
}
