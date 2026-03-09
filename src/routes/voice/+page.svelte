<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { invokeTauri, isDesktopRuntime } from '$lib/utils/desktop';

	// ─── Types ────────────────────────────────────────────────────────────────

	type ChatMessage = {
		id: string;
		sessionId: string;
		role: string;
		content: string;
		voiceInput: boolean;
		voiceOutput: boolean;
		commandType: string | null;
		actionTaken: string | null;
		createdAt: string;
	};

	type ChatSession = {
		id: string;
		name: string;
		deviceOrigin: string;
		messageCount: number;
		lastMessage: string;
		createdAt: string;
		updatedAt: string;
	};

	type PushNotification = {
		id: string;
		title: string;
		body: string;
		notificationType: string;
		payload: string;
		read: boolean;
		createdAt: string;
	};

	// ─── State ────────────────────────────────────────────────────────────────

	let sessions: ChatSession[] = [];
	let messages: ChatMessage[] = [];
	let notifications: PushNotification[] = [];
	let activeSessionId = '';
	let inputText = '';
	let sending = false;
	let voiceMode = false;
	let listening = false;
	let speaking = false;
	let messagesDiv: HTMLDivElement | undefined;

	// Web Speech API (browser types for SpeechRecognition vary by vendor)
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let recognition: any = null;
	let synthesis: SpeechSynthesis | null = null;
	let voiceEnabled = false;

	// Notification panel
	let showNotifications = false;
	let pollInterval: ReturnType<typeof setInterval> | null = null;

	// ─── Speech Setup ─────────────────────────────────────────────────────────

	function initSpeech() {
		if (typeof window === 'undefined') return;

		// Speech recognition
		const SR = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
		if (SR) {
			recognition = new SR();
			recognition!.continuous = false;
			recognition!.interimResults = false;
			recognition!.lang = 'en-US';
			// eslint-disable-next-line @typescript-eslint/no-explicit-any
			recognition!.onresult = (event: any) => {
				const transcript = event.results[0][0].transcript;
				inputText = transcript;
				listening = false;
				sendMessage(true);
			};
			recognition!.onerror = () => {
				listening = false;
			};
			recognition!.onend = () => {
				listening = false;
			};
			voiceEnabled = true;
		}

		// Speech synthesis
		if ('speechSynthesis' in window) {
			synthesis = window.speechSynthesis;
		}
	}

	function startListening() {
		if (!recognition || listening) return;
		listening = true;
		recognition.start();
	}

	function stopListening() {
		if (!recognition || !listening) return;
		recognition.stop();
		listening = false;
	}

	function speak(text: string) {
		if (!synthesis || !voiceMode) return;
		synthesis.cancel();
		const utterance = new SpeechSynthesisUtterance(text);
		utterance.rate = 1.05;
		utterance.pitch = 1.0;
		utterance.volume = 0.9;
		speaking = true;
		utterance.onend = () => (speaking = false);
		synthesis.speak(utterance);
	}

	// ─── Data Loading ─────────────────────────────────────────────────────────

	async function loadSessions() {
		if (!isDesktopRuntime()) return;
		try {
			sessions = await invokeTauri<ChatSession[]>('list_chat_sessions');
		} catch (e) {
			console.error(e);
		}
	}

	async function loadMessages(sessionId: string) {
		if (!isDesktopRuntime() || !sessionId) return;
		try {
			messages = await invokeTauri<ChatMessage[]>('list_chat_messages', { sessionId });
			scrollToBottom();
		} catch (e) {
			console.error(e);
		}
	}

	async function loadNotifications() {
		if (!isDesktopRuntime()) return;
		try {
			notifications = await invokeTauri<PushNotification[]>('list_push_notifications', {
				unreadOnly: false
			});
		} catch (e) {
			console.error(e);
		}
	}

	// ─── Messaging ────────────────────────────────────────────────────────────

	async function sendMessage(isVoice = false) {
		const text = inputText.trim();
		if (!text || sending) return;

		// Optimistic local user message
		const tempId = `temp-${Date.now()}`;
		messages = [
			...messages,
			{
				id: tempId,
				sessionId: activeSessionId,
				role: 'user',
				content: text,
				voiceInput: isVoice,
				voiceOutput: false,
				commandType: null,
				actionTaken: null,
				createdAt: new Date().toISOString()
			}
		];
		inputText = '';
		sending = true;
		scrollToBottom();

		try {
			if (isDesktopRuntime()) {
				const reply = await invokeTauri<ChatMessage>('send_chat_message', {
					req: {
						sessionId: activeSessionId || null,
						content: text,
						voiceInput: isVoice,
						deviceOrigin: isVoice ? 'voice' : 'desktop',
						model: null
					}
				});

				// Set session ID from response if new
				if (!activeSessionId) {
					activeSessionId = reply.sessionId;
					await loadSessions();
				}

				// Replace optimistic + add real reply
				messages = messages.filter((m) => m.id !== tempId);
				await loadMessages(activeSessionId);

				// Speak the response if voice mode is on
				if (voiceMode || isVoice) {
					speak(reply.content);
				}
			} else {
				// Web fallback
				await new Promise((r) => setTimeout(r, 600));
				messages = messages.filter((m) => m.id !== tempId);
				const mockReply: ChatMessage = {
					id: `web-${Date.now()}`,
					sessionId: 'web-session',
					role: 'assistant',
					content: `Hello! I'm AmitOS — your personal AI OS. I received: "${text}". In the desktop app, I can execute real voice commands and control your devices.`,
					voiceInput: false,
					voiceOutput: isVoice,
					commandType: 'none',
					actionTaken: null,
					createdAt: new Date().toISOString()
				};
				messages = [...messages, mockReply];
				if (voiceMode || isVoice) speak(mockReply.content);
			}
		} finally {
			sending = false;
			scrollToBottom();
		}
	}

	async function dismissNotification(id: string) {
		if (!isDesktopRuntime()) return;
		try {
			await invokeTauri('mark_notification_read', { id });
			notifications = notifications.map((n) => (n.id === id ? { ...n, read: true } : n));
		} catch (e) {
			console.error(e);
		}
	}

	// ─── UI Helpers ───────────────────────────────────────────────────────────

	function scrollToBottom() {
		setTimeout(() => {
			if (messagesDiv) {
				messagesDiv.scrollTop = messagesDiv.scrollHeight;
			}
		}, 50);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			sendMessage(false);
		}
	}

	function commandBadge(type: string | null) {
		if (!type || type === 'none') return null;
		const map: Record<string, string> = {
			todo: '✅ Task added',
			approve: '✅ Approved',
			deny: '🚫 Denied',
			navigate: '🔀 Navigate',
			query: '🔍 Query',
			agent_start: '▶ Agent started',
			agent_stop: '⏹ Agent stopped'
		};
		return map[type] ?? type;
	}

	$: unreadCount = notifications.filter((n) => !n.read).length;

	onMount(async () => {
		initSpeech();
		await Promise.all([loadSessions(), loadNotifications()]);
		pollInterval = setInterval(loadNotifications, 8000);

		// Create demo notification
		if (isDesktopRuntime() && notifications.length === 0) {
			try {
				await invokeTauri('create_push_notification', {
					title: 'AmitOS Voice Assistant Ready',
					body: "Say 'add to-do: buy milk' or 'approve permission' to control your devices remotely.",
					notificationType: 'info',
					payload: '{}'
				});
				await loadNotifications();
			} catch (e) { /* ignore */ }
		}
	});

	onDestroy(() => {
		if (pollInterval) clearInterval(pollInterval);
		if (recognition && listening) recognition.stop();
		if (synthesis) synthesis.cancel();
	});
</script>

<section class="space-y-6">
	<!-- Header -->
	<div class="rounded-[2rem] border border-white/10 bg-slate-950/50 p-8 shadow-2xl shadow-cyan-950/20 backdrop-blur">
		<div class="flex items-start justify-between gap-4 flex-wrap">
			<div>
				<p class="text-sm uppercase tracking-[0.35em] text-cyan-300/80">Voice + Chat Assistant</p>
				<h1 class="mt-4 text-4xl font-semibold tracking-tight text-white sm:text-5xl">
					AmitOS Voice Control
				</h1>
				<p class="mt-4 text-base leading-7 text-slate-300 max-w-xl">
					Real-time voice commands + chat from your phone or desktop. Say "add to-do: buy milk" or "approve the current permission". Remote control while you're out.
				</p>
			</div>

			<div class="flex flex-col items-end gap-3">
				<!-- Notifications bell -->
				<button
					type="button"
					on:click={() => (showNotifications = !showNotifications)}
					class="relative rounded-full border border-white/15 bg-white/5 p-3 text-white hover:bg-white/10"
				>
					🔔
					{#if unreadCount > 0}
						<span class="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-amber-400 text-[10px] font-bold text-slate-950">
							{unreadCount}
						</span>
					{/if}
				</button>

				<!-- Voice mode toggle -->
				<div class="flex cursor-pointer items-center gap-2 text-sm">
					<span class="text-slate-400">Voice output</span>
					<button
						type="button"
						role="switch"
						aria-checked={voiceMode}
						on:click={() => (voiceMode = !voiceMode)}
						class="relative h-6 w-11 rounded-full transition {voiceMode ? 'bg-cyan-500' : 'bg-slate-700'} cursor-pointer border-0 p-0"
					>
						<span class="absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform {voiceMode ? 'translate-x-5' : 'translate-x-0'}"></span>
					</button>
				</div>

				{#if !voiceEnabled}
					<p class="text-xs text-slate-600">Voice not supported in this browser</p>
				{/if}
			</div>
		</div>
	</div>

	<!-- Notification Panel -->
	{#if showNotifications}
		<div class="rounded-3xl border border-amber-400/20 bg-amber-500/5 p-6 backdrop-blur">
			<div class="flex items-center justify-between mb-4">
				<h2 class="text-sm font-semibold text-white">Notifications</h2>
				<button type="button" on:click={() => (showNotifications = false)} class="text-slate-500 hover:text-white text-sm">✕</button>
			</div>
			{#if notifications.length === 0}
				<p class="text-sm text-slate-500">No notifications yet.</p>
			{:else}
				<div class="space-y-3">
					{#each notifications as notif}
						<div class="flex items-start gap-3 rounded-2xl border {notif.read ? 'border-white/5 bg-white/2' : 'border-amber-400/20 bg-amber-500/5'} p-4">
							<span class="mt-0.5 text-lg">
								{notif.notificationType === 'permission_request' ? '🔐' :
								 notif.notificationType === 'task_complete' ? '✅' :
								 notif.notificationType === 'error' ? '❌' : 'ℹ️'}
							</span>
							<div class="flex-1">
								<p class="text-sm font-medium text-white">{notif.title}</p>
								<p class="text-xs text-slate-400 mt-1">{notif.body}</p>
								<p class="text-xs text-slate-600 mt-1">{new Date(notif.createdAt).toLocaleString()}</p>
							</div>
							{#if !notif.read}
								<button
									type="button"
									on:click={() => dismissNotification(notif.id)}
									class="text-xs text-slate-500 hover:text-white"
								>
									✓
								</button>
							{/if}
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}

	<div class="grid gap-6 lg:grid-cols-[240px_1fr]">
		<!-- Session Sidebar -->
		<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-4 backdrop-blur">
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-xs font-semibold text-slate-400 uppercase tracking-wider">Sessions</h2>
				<button
					type="button"
					on:click={() => {
						activeSessionId = '';
						messages = [];
					}}
					class="text-xs text-cyan-400 hover:text-cyan-300"
				>
					+ New
				</button>
			</div>
			<div class="space-y-2">
				<button
					type="button"
					on:click={() => {
						activeSessionId = '';
						messages = [];
					}}
					class="w-full rounded-xl px-3 py-2.5 text-left text-xs transition {!activeSessionId
						? 'bg-cyan-400/15 text-cyan-100'
						: 'text-slate-400 hover:bg-white/5 hover:text-white'}"
				>
					New conversation
				</button>
				{#each sessions as session}
					<button
						type="button"
						on:click={async () => {
							activeSessionId = session.id;
							await loadMessages(session.id);
						}}
						class="w-full rounded-xl px-3 py-2.5 text-left text-xs transition {activeSessionId ===
						session.id
							? 'bg-cyan-400/15 text-cyan-100'
							: 'text-slate-400 hover:bg-white/5 hover:text-white'}"
					>
						<p class="font-medium truncate">{session.name}</p>
						<p class="text-slate-600 truncate mt-0.5">{session.lastMessage || '...'}</p>
						<p class="text-[10px] text-slate-700 mt-0.5">{session.deviceOrigin} · {session.messageCount} msgs</p>
					</button>
				{/each}
			</div>
		</div>

		<!-- Chat Window -->
		<div class="flex flex-col rounded-3xl border border-white/10 bg-slate-950/45 backdrop-blur overflow-hidden" style="height: 600px;">
			<!-- Messages -->
			<div
				bind:this={messagesDiv}
				class="flex-1 overflow-y-auto p-5 space-y-4"
			>
				{#if messages.length === 0}
					<!-- Welcome screen -->
					<div class="flex h-full flex-col items-center justify-center gap-6 py-8">
						<div class="text-5xl">🎙️</div>
						<div class="text-center max-w-sm">
							<p class="text-lg font-semibold text-white">AmitOS Voice + Chat</p>
							<p class="mt-2 text-sm text-slate-400">Your personal AI OS — remote control via voice or text from anywhere</p>
						</div>
						<div class="grid grid-cols-2 gap-3 w-full max-w-sm">
							{#each [
								{ text: 'add to-do: buy milk', icon: '✅' },
								{ text: 'approve the current permission', icon: '🔐' },
								{ text: "what's my memory status?", icon: '🧠' },
								{ text: 'start agent session', icon: '🤖' }
							] as suggestion}
								<button
									type="button"
									on:click={() => {
										inputText = suggestion.text;
										sendMessage(false);
									}}
									class="rounded-2xl border border-white/8 bg-white/3 p-3 text-left hover:border-white/15 transition"
								>
									<span class="text-lg">{suggestion.icon}</span>
									<p class="text-xs text-slate-400 mt-1">{suggestion.text}</p>
								</button>
							{/each}
						</div>
					</div>
				{:else}
					{#each messages as msg}
						<div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'} gap-2">
							{#if msg.role !== 'user'}
								<div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-violet-500 to-cyan-500 text-xs mt-1">
									AI
								</div>
							{/if}
							<div class="max-w-[80%]">
								<div
									class="rounded-2xl px-4 py-3 text-sm {msg.role === 'user'
										? 'bg-cyan-500/20 border border-cyan-400/20 text-white rounded-br-sm'
										: 'bg-slate-800/60 border border-white/8 text-slate-200 rounded-bl-sm'}"
								>
									{#if msg.voiceInput}
										<span class="text-xs text-cyan-400/60 mr-1">🎙️</span>
									{/if}
									{msg.content}
								</div>
								{#if commandBadge(msg.commandType)}
									<span class="mt-1 inline-block rounded-full border border-green-400/20 bg-green-500/10 px-2 py-0.5 text-[10px] text-green-400">
										{commandBadge(msg.commandType)}
									</span>
								{/if}
								{#if msg.actionTaken}
									<p class="mt-1 text-[10px] text-slate-600">{msg.actionTaken}</p>
								{/if}
							</div>
							{#if msg.role === 'user'}
								<div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-slate-700 text-xs mt-1">
									You
								</div>
							{/if}
						</div>
					{/each}
					{#if sending}
						<div class="flex justify-start gap-2">
							<div class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-violet-500 to-cyan-500 text-xs">
								AI
							</div>
							<div class="rounded-2xl rounded-bl-sm bg-slate-800/60 border border-white/8 px-4 py-3">
								<div class="flex gap-1">
									<span class="h-1.5 w-1.5 rounded-full bg-slate-500 animate-bounce" style="animation-delay:0ms"></span>
									<span class="h-1.5 w-1.5 rounded-full bg-slate-500 animate-bounce" style="animation-delay:150ms"></span>
									<span class="h-1.5 w-1.5 rounded-full bg-slate-500 animate-bounce" style="animation-delay:300ms"></span>
								</div>
							</div>
						</div>
					{/if}
				{/if}
			</div>

			<!-- Input Bar -->
			<div class="border-t border-white/8 p-4">
				<div class="flex items-end gap-3">
					<!-- Voice button -->
					{#if voiceEnabled}
						<button
							type="button"
							on:click={listening ? stopListening : startListening}
							class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full transition {listening
								? 'bg-rose-500 text-white animate-pulse'
								: 'border border-white/15 bg-white/5 text-slate-400 hover:text-white'}"
							title={listening ? 'Stop listening' : 'Start voice input'}
						>
							🎙️
						</button>
					{/if}

					<textarea
						bind:value={inputText}
						on:keydown={handleKeydown}
						rows={1}
						disabled={sending}
						placeholder={listening ? '🎙️ Listening...' : 'Type a message or voice command...'}
						class="flex-1 resize-none rounded-2xl border border-white/10 bg-slate-900/60 px-4 py-2.5 text-sm text-white placeholder-slate-500 focus:border-cyan-400/40 focus:outline-none disabled:opacity-50"
						style="min-height: 40px; max-height: 120px;"
					></textarea>

					<button
						type="button"
						on:click={() => sendMessage(false)}
						disabled={sending || !inputText.trim()}
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-cyan-500 to-violet-500 text-white shadow-lg shadow-cyan-500/20 disabled:opacity-40"
					>
						{#if sending}
							<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
								<circle cx="12" cy="12" r="10" stroke-opacity="0.25" />
								<path d="M12 2a10 10 0 0 1 10 10" />
							</svg>
						{:else}
							↑
						{/if}
					</button>
				</div>

				<div class="mt-2 flex items-center justify-between text-[10px] text-slate-700">
					<span>Enter to send · Shift+Enter for newline · 🎙️ for voice</span>
					{#if speaking}
						<span class="text-cyan-500 animate-pulse">🔊 Speaking...</span>
					{/if}
				</div>
			</div>
		</div>
	</div>

	<!-- Voice Command Reference -->
	<div class="rounded-3xl border border-white/10 bg-slate-950/45 p-6 backdrop-blur">
		<h2 class="text-sm font-semibold text-white mb-4">Voice Command Reference</h2>
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			{#each [
				{ category: 'Tasks', icon: '✅', examples: ['"add to-do: buy milk"', '"add task: call dentist"', '"remind me to submit report"'] },
				{ category: 'Permissions', icon: '🔐', examples: ['"approve the current permission"', '"deny the agent action"', '"approve Ralph loop"'] },
				{ category: 'Navigation', icon: '🔀', examples: ['"open settings"', '"go to deploy"', '"show me workflows"'] },
				{ category: 'Agent Control', icon: '🤖', examples: ['"start agent session"', '"stop agent"', '"pause all tasks"'] },
				{ category: 'Status Queries', icon: '🔍', examples: ['"what\'s my memory status?"', '"how many tasks pending?"', '"agent status?"'] },
				{ category: 'Greetings', icon: '👋', examples: ['"hello AmitOS"', '"help"', '"what can you do?"'] }
			] as group}
				<div class="rounded-2xl border border-white/8 bg-white/3 p-4">
					<div class="flex items-center gap-2 mb-3">
						<span class="text-lg">{group.icon}</span>
						<h3 class="text-xs font-semibold text-white">{group.category}</h3>
					</div>
					{#each group.examples as example}
						<button
							type="button"
							on:click={() => {
								inputText = example.replace(/"/g, '');
								sendMessage(false);
							}}
							class="block w-full text-left rounded-lg px-2 py-1.5 text-xs text-slate-500 hover:bg-white/5 hover:text-slate-300 transition mb-0.5 font-mono"
						>
							{example}
						</button>
					{/each}
				</div>
			{/each}
		</div>
	</div>
</section>
