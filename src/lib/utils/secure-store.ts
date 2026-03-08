import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

export const KEY_FIELDS = [
	'ANTHROPIC_API_KEY',
	'OPENAI_API_KEY',
	'GROK_API_KEY',
	'GEMINI_API_KEY',
	'PERPLEXICA_KEYS'
] as const;

export type KeyField = (typeof KEY_FIELDS)[number];
export type KeyMap = Record<KeyField, string>;

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

export async function loadKeys(): Promise<KeyMap> {
	const defaults = createEmptyKeyMap();
	if (!isDesktopRuntime()) {
		return defaults;
	}

	const [{ Stronghold }, config] = await Promise.all([
		import('@tauri-apps/plugin-stronghold'),
		invokeTauri<SecureStoreConfig>('get_secure_store_config')
	]);
	const stronghold = await Stronghold.load(config.vaultPath, config.vaultPassword);
	const client = await getClient(stronghold, config.clientName);
	const store = client.getStore();

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

	const [{ Stronghold }, config] = await Promise.all([
		import('@tauri-apps/plugin-stronghold'),
		invokeTauri<SecureStoreConfig>('get_secure_store_config')
	]);
	const stronghold = await Stronghold.load(config.vaultPath, config.vaultPassword);
	const client = await getClient(stronghold, config.clientName);
	const store = client.getStore();

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
