import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

// Legacy fixed key fields (kept for backward compat)
export const KEY_FIELDS = [
	'ANTHROPIC_API_KEY',
	'OPENAI_API_KEY',
	'GROK_API_KEY',
	'GEMINI_API_KEY',
	'PERPLEXICA_KEYS'
] as const;

export type KeyField = (typeof KEY_FIELDS)[number];
export type KeyMap = Record<KeyField, string>;

// Dynamic key map for any string key
export type DynamicKeyMap = Record<string, string>;

type SecureStoreConfig = {
	vaultPath: string;
	clientName: string;
	vaultPassword: string;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function createEmptyKeyMap(): KeyMap {
	return {
		ANTHROPIC_API_KEY: '',
		OPENAI_API_KEY: '',
		GROK_API_KEY: '',
		GEMINI_API_KEY: '',
		PERPLEXICA_KEYS: ''
	};
}

async function getStronghold(): Promise<{ stronghold: any; client: any; store: any; config: SecureStoreConfig }> {
	const [{ Stronghold }, config] = await Promise.all([
		import('@tauri-apps/plugin-stronghold'),
		invokeTauri<SecureStoreConfig>('get_secure_store_config')
	]);
	const stronghold = await Stronghold.load(config.vaultPath, config.vaultPassword);
	const client = await getClient(stronghold, config.clientName);
	const store = client.getStore();
	return { stronghold, client, store, config };
}

export async function loadKeys(): Promise<KeyMap> {
	const defaults = createEmptyKeyMap();
	if (!isDesktopRuntime()) {
		return defaults;
	}

	const { stronghold, store } = await getStronghold();

	try {
		for (const field of KEY_FIELDS) {
			const storedValue = await store.get(field);
			defaults[field] = decodeValue(storedValue);
		}
		return defaults;
	} finally {
		await stronghold.unload();
	}
}

export async function saveKeys(values: KeyMap): Promise<void> {
	if (!isDesktopRuntime()) {
		throw new Error('Secure storage is only available inside the RalphHub desktop runtime.');
	}

	const { stronghold, store } = await getStronghold();

	try {
		for (const field of KEY_FIELDS) {
			const value = values[field].trim();
			if (value) {
				await store.insert(field, Array.from(encoder.encode(value)));
			} else {
				await store.remove(field);
			}
		}
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

// ─── Dynamic key operations for any provider env var ─────────────────────────

export async function loadDynamicKeys(keyNames: string[]): Promise<DynamicKeyMap> {
	const result: DynamicKeyMap = {};
	for (const k of keyNames) result[k] = '';

	if (!isDesktopRuntime() || keyNames.length === 0) return result;

	const { stronghold, store } = await getStronghold();
	try {
		for (const key of keyNames) {
			const storedValue = await store.get(key);
			result[key] = decodeValue(storedValue);
		}
		return result;
	} finally {
		await stronghold.unload();
	}
}

export async function saveDynamicKey(keyName: string, value: string): Promise<void> {
	if (!isDesktopRuntime()) {
		throw new Error('Secure storage is only available inside the RalphHub desktop runtime.');
	}

	const { stronghold, store } = await getStronghold();
	try {
		const trimmed = value.trim();
		if (trimmed) {
			await store.insert(keyName, Array.from(encoder.encode(trimmed)));
		} else {
			await store.remove(keyName);
		}
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

export async function saveDynamicKeys(keyMap: DynamicKeyMap): Promise<void> {
	if (!isDesktopRuntime()) {
		throw new Error('Secure storage is only available inside the RalphHub desktop runtime.');
	}

	const { stronghold, store } = await getStronghold();
	try {
		for (const [key, value] of Object.entries(keyMap)) {
			const trimmed = value.trim();
			if (trimmed) {
				await store.insert(key, Array.from(encoder.encode(trimmed)));
			} else {
				await store.remove(key);
			}
		}
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

export async function hasKey(keyName: string): Promise<boolean> {
	if (!isDesktopRuntime()) return false;
	const { stronghold, store } = await getStronghold();
	try {
		const val = await store.get(keyName);
		return !!val && decodeValue(val).length > 0;
	} catch {
		return false;
	} finally {
		await stronghold.unload();
	}
}

export async function deleteKey(keyName: string): Promise<void> {
	if (!isDesktopRuntime()) return;
	const { stronghold, store } = await getStronghold();
	try {
		await store.remove(keyName);
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

async function getClient(stronghold: { loadClient(name: string): Promise<any>; createClient(name: string): Promise<any> }, clientName: string) {
	try {
		return await stronghold.loadClient(clientName);
	} catch {
		return await stronghold.createClient(clientName);
	}
}

function decodeValue(value: Uint8Array | number[] | null): string {
	if (!value?.length) {
		return '';
	}
	return decoder.decode(new Uint8Array(value));
}
