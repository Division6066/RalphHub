import { isDesktopRuntime, invokeTauri } from '$lib/utils/desktop';

export const KEY_FIELDS = [
	'ANTHROPIC_API_KEY',
	'OPENAI_API_KEY',
	'GROK_API_KEY',
	'GEMINI_API_KEY',
	'GLM_API_KEY',
	'PERPLEXICA_KEYS',
	'OLLAMA_API_KEY',
	'OLLAMA_CLOUD_API_KEY',
	'NOTION_API_KEY',
	'GITHUB_TOKEN',
	'HF_TOKEN'
] as const;

export type KeyField = (typeof KEY_FIELDS)[number];
export type KeyMap = Record<KeyField, string>;

export const KEY_LABELS: Record<KeyField, { label: string; provider: string; url: string }> = {
	ANTHROPIC_API_KEY: { label: 'Anthropic API Key', provider: 'Anthropic', url: 'https://console.anthropic.com/settings/keys' },
	OPENAI_API_KEY: { label: 'OpenAI API Key', provider: 'OpenAI', url: 'https://platform.openai.com/api-keys' },
	GROK_API_KEY: { label: 'Grok API Key', provider: 'xAI Grok', url: 'https://console.x.ai' },
	GEMINI_API_KEY: { label: 'Gemini API Key', provider: 'Google Gemini', url: 'https://aistudio.google.com/app/apikey' },
	GLM_API_KEY: { label: 'GLM API Key', provider: 'Zhipu AI GLM', url: 'https://open.bigmodel.cn/usercenter/apikeys' },
	PERPLEXICA_KEYS: { label: 'Perplexica Keys', provider: 'Perplexica', url: 'https://github.com/ItzCrazyKns/Perplexica' },
	OLLAMA_API_KEY: { label: 'Ollama Local Endpoint', provider: 'Ollama Local', url: 'http://localhost:11434' },
	OLLAMA_CLOUD_API_KEY: { label: 'Ollama Cloud API Key', provider: 'Ollama Cloud', url: 'https://ollama.ai/cloud' },
	NOTION_API_KEY: { label: 'Notion Integration Token', provider: 'Notion', url: 'https://www.notion.so/profile/integrations' },
	GITHUB_TOKEN: { label: 'GitHub Personal Token', provider: 'GitHub', url: 'https://github.com/settings/tokens' },
	HF_TOKEN: { label: 'Hugging Face Token', provider: 'Hugging Face', url: 'https://huggingface.co/settings/tokens' }
};

type SecureStoreConfig = {
	vaultPath: string;
	clientName: string;
	vaultPassword: string;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function createEmptyKeyMap(): KeyMap {
	return Object.fromEntries(KEY_FIELDS.map((k) => [k, ''])) as KeyMap;
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
