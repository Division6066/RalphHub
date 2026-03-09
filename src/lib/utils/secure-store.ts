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

// ─── Stronghold helper ────────────────────────────────────────────────────────

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

// ─── Legacy fixed-key operations ─────────────────────────────────────────────

export async function loadKeys(): Promise<KeyMap> {
	const defaults = createEmptyKeyMap();

	if (!isDesktopRuntime()) {
		// Browser fallback: load from localStorage
		for (const field of KEY_FIELDS) {
			const v = localStorage.getItem(`amitos_key_${field}`);
			if (v) defaults[field] = v;
		}
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
		// Browser fallback: save to localStorage
		for (const field of KEY_FIELDS) {
			const v = (values[field] ?? '').trim();
			if (v) {
				localStorage.setItem(`amitos_key_${field}`, v);
			} else {
				localStorage.removeItem(`amitos_key_${field}`);
			}
		}
		return;
	}

	const { stronghold, store } = await getStronghold();
	try {
		for (const field of KEY_FIELDS) {
			const value = (values[field] ?? '').trim();
			if (value) {
				await store.insert(field, Array.from(encoder.encode(value)));
			} else {
				try { await store.remove(field); } catch {}
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

	if (!isDesktopRuntime() || keyNames.length === 0) {
		// Browser fallback
		for (const k of keyNames) {
			result[k] = localStorage.getItem(`amitos_key_${k}`) ?? '';
		}
		return result;
	}

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
		const trimmed = value.trim();
		if (trimmed) {
			localStorage.setItem(`amitos_key_${keyName}`, trimmed);
		} else {
			localStorage.removeItem(`amitos_key_${keyName}`);
		}
		return;
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
		for (const [key, value] of Object.entries(keyMap)) {
			const trimmed = value.trim();
			if (trimmed) {
				localStorage.setItem(`amitos_key_${key}`, trimmed);
			} else {
				localStorage.removeItem(`amitos_key_${key}`);
			}
		}
		return;
	}

	const { stronghold, store } = await getStronghold();
	try {
		for (const [key, value] of Object.entries(keyMap)) {
			const trimmed = value.trim();
			if (trimmed) {
				await store.insert(key, Array.from(encoder.encode(trimmed)));
			} else {
				try { await store.remove(key); } catch {}
			}
		}
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

export async function hasKey(keyName: string): Promise<boolean> {
	if (!isDesktopRuntime()) {
		const v = localStorage.getItem(`amitos_key_${keyName}`);
		return !!v && v.length > 0;
	}
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
	if (!isDesktopRuntime()) {
		localStorage.removeItem(`amitos_key_${keyName}`);
		return;
	}
	const { stronghold, store } = await getStronghold();
	try {
		await store.remove(keyName);
		await stronghold.save();
	} finally {
		await stronghold.unload();
	}
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

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
