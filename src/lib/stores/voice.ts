import { writable, derived } from 'svelte/store';

export type VoiceState = {
	globalEnabled: boolean;
	sttProvider: string;
	ttsProvider: string;
	whisperInstalled: boolean;
	piperInstalled: boolean;
	offlineFallback: boolean;
	recording: boolean;
	speaking: boolean;
	lastTranscript: string;
	lastResponse: string;
};

const defaultVoiceState: VoiceState = {
	globalEnabled: false,
	sttProvider: 'none',
	ttsProvider: 'none',
	whisperInstalled: false,
	piperInstalled: false,
	offlineFallback: false,
	recording: false,
	speaking: false,
	lastTranscript: '',
	lastResponse: ''
};

export const voiceStore = writable<VoiceState>(defaultVoiceState);

export const voiceReady = derived(
	voiceStore,
	($v) => $v.globalEnabled && ($v.whisperInstalled || $v.piperInstalled)
);

export function enableVoice() {
	voiceStore.update((s) => ({ ...s, globalEnabled: true }));
}

export function disableVoice() {
	voiceStore.update((s) => ({ ...s, globalEnabled: false }));
}

export function setVoiceConfig(config: Partial<VoiceState>) {
	voiceStore.update((s) => ({ ...s, ...config }));
}
