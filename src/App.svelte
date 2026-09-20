<script lang="ts">
  import { onMount } from 'svelte';
  import { Clipboard, FilePlus, KeyRound, LoaderCircle, RefreshCw, Settings, Sparkles, X } from 'lucide-svelte';
  import ReplyCard from './components/ReplyCard.svelte';
  import { clearDraft, defaults, desktop, generate, getConfig, loadDraft, paste, saveDraft, type Config, type Reply } from './lib/api';

  let source = $state('');
  let direction = $state('');
  let preset = $state('丁寧に');
  let greeting = $state(true);
  let count = $state(1);
  let decorations = $state<string[]>([]);
  let config = $state<Config>(defaults);
  let replies = $state<Reply[]>([]);
  let busy = $state(false);
  let initializing = $state(true);
  let settings = $state(false);
  let error = $state('');
  let notice = $state('');
  let savedTo = $state<string | null>(null);
  let confirmNew = $state(false);
  let sourceInput: HTMLTextAreaElement;
  let keyLoaded = $state(false);
  let draftReady = $state(false);
  let restored = false;
  // New で破棄するたびに進める。非同期に source / replies を書き戻す経路
  // (復元・クリップボード貼り付け・生成完了) は、開始時の世代と一致する時だけ反映する。
  // そうしないと、破棄した後に遅れて届いた元メールがフォームとディスクに戻る
  let generation = 0;
  // ブラウザプレビューでは生成できないので、キー未設定でもブロックしない
  let locked = $derived(desktop && keyLoaded && !config.configured);
  let words = $derived(source.trim() ? source.trim().split(/\s+/).length : 0);
  const suggestions = ['やんわり断る', '日程を調整する', '感謝を伝える'];
  const mac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.userAgent);
  const keys = mac ? { mod: '⌘', enter: '⌘↵', comma: '⌘,', paste: '⌘V' } : { mod: 'Ctrl', enter: 'Ctrl+Enter', comma: 'Ctrl+,', paste: 'Ctrl+V' };

  async function reload(initial = false) {
    const gen = generation;
    initializing = true;
    error = '';
    try {
      config = await getConfig();
      // config の既定値を入れるのは初回だけ。Settings の Reload で入れ直すと、
      // ユーザーが選んだ greeting / 件数を黙って巻き戻す (preset と同じ扱いに揃える)
      if (initial && !restored) {
        greeting = config.greeting.enabled;
        count = config.general.default_n;
      }
      decorations = decorations
        .filter((t) => config.decorations.some((d) => d.title === t))
        .filter((t, i, list) => !groupOf(t) || !list.slice(i + 1).some((u) => groupOf(u) === groupOf(t)));
      if (!config.preset_prompts.some((p) => p.name === preset)) preset = config.preset_prompts[0]?.name ?? '';
      if (initial && gen === generation && config.configured && config.general.clipboard_auto_paste && !source) await pasteEmail();
    } catch (e) {
      error = String(e);
    } finally {
      initializing = false;
      keyLoaded = true;
    }
  }
  async function restoreDraft() {
    const gen = generation;
    try {
      const draft = await loadDraft();
      if (!draft || gen !== generation) return;
      if (typeof draft.source === 'string') source = draft.source;
      if (typeof draft.direction === 'string') direction = draft.direction;
      if (typeof draft.preset === 'string') preset = draft.preset;
      if (typeof draft.greeting === 'boolean') greeting = draft.greeting;
      if (typeof draft.count === 'number' && [1, 2, 3].includes(draft.count)) count = draft.count;
      if (Array.isArray(draft.decorations)) decorations = draft.decorations.filter((t) => typeof t === 'string');
      if (Array.isArray(draft.replies)) {
        replies = draft.replies.filter(
          (r): r is Reply =>
            !!r && typeof r.reply_text === 'string' && typeof r.subject_line === 'string' && typeof r.tone_used === 'string',
        );
      }
      restored = true;
      notice = 'Restored your previous draft';
    } catch {
      notice = 'Could not restore the previous draft';
    }
  }
  onMount(async () => {
    await restoreDraft();
    draftReady = true;
    await reload(true);
  });

  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let savePending: Promise<unknown> = Promise.resolve();
  $effect(() => {
    const draft = { source, direction, preset, greeting, count, decorations: $state.snapshot(decorations), replies: $state.snapshot(replies) };
    if (!draftReady) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      const empty = !draft.source && !draft.direction && !draft.replies.length;
      // 直列化する。代入で上書きすると、先行する保存/削除との着地順が保証されない
      savePending = savePending
        .catch(() => {})
        .then(() => (empty ? clearDraft() : saveDraft(draft)))
        .catch(() => (notice = 'Could not save the draft'));
    }, 400);
  });

  async function pasteEmail() {
    const gen = generation;
    try {
      const text = await paste();
      if (gen !== generation) return;
      if (text?.trim()) {
        source = text;
        notice = 'Pasted from clipboard';
      } else {
        error = 'The clipboard has no text.';
      }
    } catch {
      if (gen !== generation) return;
      error = `Could not read the clipboard. Paste into the field with ${keys.paste}.`;
    }
  }

  async function compose() {
    if (busy || initializing || locked || !source.trim()) return;
    // 確認バーを出したまま生成を始めると、生成中の Clear の後に返信が届いて
    // 下書きが作り直される
    confirmNew = false;
    const gen = generation;
    busy = true;
    error = '';
    notice = '';
    try {
      const result = await generate(source, direction, preset, greeting, count, decorations);
      if (gen !== generation) return;
      replies = result.variants;
      savedTo = result.saved_to;
      notice = result.warnings.length
        ? result.warnings.join(' ')
        : result.copied
          ? `${result.variants.length} replies generated · First reply copied`
          : `${result.variants.length} replies generated`;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  const groupOf = (title: string) => config.decorations.find((d) => d.title === title)?.group?.trim() || null;

  function toggleDecoration(title: string) {
    if (decorations.includes(title)) {
      decorations = decorations.filter((t) => t !== title);
      return;
    }
    const group = groupOf(title);
    decorations = [...decorations.filter((t) => !group || groupOf(t) !== group), title];
  }

  function newDraft() {
    if ((source || direction || replies.length) && !confirmNew) {
      confirmNew = true;
      return;
    }
    source = '';
    direction = '';
    replies = [];
    error = '';
    notice = '';
    savedTo = null;
    confirmNew = false;
    // debounce を待たずに消す。直後に終了・クラッシュすると元メールを含む
    // draft.json が残り、次回起動で復元されてしまう。
    // 進行中の保存を待ってから消すのは、既に飛んだ save_draft が clear_draft の
    // 後に着地して draft.json が復活するのを防ぐため
    clearTimeout(saveTimer);
    generation += 1;
    const clear = () => clearDraft().catch(() => (notice = 'Could not clear the saved draft'));
    // 1 回目は即時 (直後に終了・クラッシュしても消えている)、2 回目は進行中の保存が
    // 後から着地する分を消すため。clear_draft は不在を無視するので二重でも害はない
    void clear();
    savePending = savePending.catch(() => {}).then(clear);
    sourceInput?.focus();
  }

  function keyboard(event: KeyboardEvent) {
    if (locked) return;
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      void compose();
    }
    if ((event.metaKey || event.ctrlKey) && event.key === ',') {
      event.preventDefault();
      settings = true;
    }
    if (event.key === 'Escape') {
      settings = false;
      confirmNew = false;
    }
  }

  const toolButton =
    'flex h-7 items-center gap-1.5 rounded-md px-2 text-[12px] text-fg hover:bg-hover disabled:hover:bg-transparent';
  const paneHeader =
    'flex h-8 shrink-0 items-center gap-2 border-b border-line px-3 text-[11px] font-semibold tracking-wide text-muted uppercase';
  const fieldLabel = 'mb-1.5 block text-[12px] font-medium text-fg';
  const field =
    'w-full rounded-md border border-field-line bg-field px-2 text-[13px] text-fg focus:border-accent focus:outline-none focus:ring-2 focus:ring-accent/30';
</script>

<svelte:window onkeydown={keyboard} />

<div class="flex h-screen flex-col bg-window text-fg">
  <!-- Overlay タイトルバー: 左 80px は信号機ボタンの領域 -->
  <header data-tauri-drag-region class="flex h-10 shrink-0 items-center gap-1 border-b border-line bg-toolbar pr-2 pl-20">
    <span data-tauri-drag-region class="mr-3 text-[13px] font-semibold">Toneweave</span>
    <button class={toolButton} data-annotate="button-new-draft" disabled={busy || initializing || !draftReady} onclick={newDraft} title="New draft">
      <FilePlus size={15} />New
    </button>
    <button class={toolButton} data-annotate="button-paste" disabled={busy} onclick={pasteEmail} title="Paste email from clipboard">
      <Clipboard size={15} />Paste
    </button>
    <div data-tauri-drag-region class="h-full flex-1"></div>
    <button
      class="flex h-7 items-center gap-1.5 rounded-md bg-accent px-3 text-[12px] font-medium text-on-accent hover:bg-accent-hover disabled:hover:bg-accent"
      data-annotate="button-generate-toolbar"
      disabled={!source.trim() || busy || initializing}
      onclick={compose}
      title="Generate replies ({keys.enter})"
    >
      {#if busy}<LoaderCircle size={14} class="spin" />Generating…{:else}<Sparkles size={14} />Generate{/if}
    </button>
    <button class="{toolButton} ml-1 w-7 justify-center px-0" data-annotate="button-settings" aria-label="Settings" title="Settings ({keys.comma})" onclick={() => (settings = true)}>
      <Settings size={16} />
    </button>
  </header>

  {#if !desktop}
    <div class="flex h-7 shrink-0 items-center border-b border-line bg-info-bg px-3 text-[12px]">
      Browser preview. Run <code class="mx-1 font-mono">pnpm tauri dev</code> to generate replies.
    </div>
  {/if}
  {#if confirmNew}
    <div class="flex h-9 shrink-0 items-center gap-2 border-b border-line bg-info-bg px-3 text-[12px]" role="alert">
      <span class="flex-1">Clear the current draft? Saved result files are kept.</span>
      <button class="h-6 rounded border border-field-line bg-field px-2.5 hover:bg-hover" data-annotate="button-confirm-new" disabled={busy} onclick={newDraft}>Clear</button>
      <button class="h-6 rounded border border-field-line bg-field px-2.5 hover:bg-hover" data-annotate="button-cancel-new" onclick={() => (confirmNew = false)}>Cancel</button>
    </div>
  {/if}
  {#if error}
    <div class="flex min-h-9 shrink-0 items-center gap-2 border-b border-line bg-danger-bg px-3 py-1.5 text-[12px] text-danger" role="alert">
      <span class="selectable flex-1 break-all">{error}</span>
      <button class="flex h-6 w-6 items-center justify-center rounded hover:bg-hover" data-annotate="button-dismiss-error" aria-label="Dismiss error" onclick={() => (error = '')}>
        <X size={14} />
      </button>
    </div>
  {/if}

  <div class="flex min-h-0 flex-1">
    <section class="flex min-w-0 flex-1 flex-col border-r border-line" aria-label="Original email">
      <div class={paneHeader}>Original email</div>
      <textarea
        bind:this={sourceInput}
        bind:value={source}
        class="min-h-0 flex-1 resize-none border-0 bg-window p-3 text-[13px] leading-relaxed text-fg placeholder:text-muted/70 focus:outline-none"
        data-annotate="input-source"
        disabled={busy}
        maxlength={100000}
        placeholder={`Paste the email you want to reply to (${keys.paste})`}
        spellcheck="false"
      ></textarea>
      <div class="flex h-6 shrink-0 items-center justify-end gap-3 border-t border-line bg-pane px-3 text-[11px] text-muted tabular-nums">
        <span>{source.length.toLocaleString()} chars</span>
        <span>{words} words</span>
      </div>
    </section>

    <section class="flex min-w-0 flex-1 flex-col border-r border-line bg-pane" aria-label="Options">
      <div class={paneHeader}>Options</div>
      <div class="flex min-h-0 flex-1 flex-col gap-5 overflow-auto p-3">
        <div>
          <label class={fieldLabel} for="direction">Direction</label>
          <textarea
            id="direction"
            bind:value={direction}
            class="{field} h-60 min-h-[84px] resize-y py-1.5 leading-relaxed"
            data-annotate="input-direction"
            disabled={busy}
            maxlength={8000}
            placeholder="e.g. やんわり断りつつ、次の機会につなげたい"
            rows="10"
          ></textarea>
          <div class="mt-2 flex flex-wrap gap-1.5">
            {#each suggestions as suggestion}
              <button
                class="h-6 rounded border border-field-line bg-field px-2 text-[12px] hover:bg-hover"
                data-annotate="button-suggestion"
                disabled={busy}
                onclick={() => (direction = suggestion)}
              >{suggestion}</button>
            {/each}
          </div>
        </div>

        <div>
          <label class={fieldLabel} for="preset">Tone</label>
          <select id="preset" bind:value={preset} class="{field} h-7" data-annotate="select-preset" disabled={busy || initializing}>
            {#each config.preset_prompts as item}<option value={item.name}>{item.name}</option>{/each}
          </select>
        </div>

        <div>
          <span class={fieldLabel} id="count-label">Replies</span>
          <div class="flex h-7 w-fit overflow-hidden rounded-md border border-field-line" role="radiogroup" aria-labelledby="count-label">
            {#each [1, 2, 3] as n}
              <button
                class="w-10 text-[12px] not-first:border-l not-first:border-field-line {count === n ? 'bg-accent text-on-accent' : 'bg-field hover:bg-hover'}"
                data-annotate="button-count-{n}"
                role="radio"
                aria-checked={count === n}
                disabled={busy}
                onclick={() => (count = n)}
              >{n}</button>
            {/each}
          </div>
        </div>

        <div>
          <span class={fieldLabel} id="decorations-label">Decorations</span>
          {#if config.decorations.length}
            <div class="flex flex-wrap gap-1.5" role="group" aria-labelledby="decorations-label">
              {#each config.decorations as decoration (decoration.title)}
                {@const active = decorations.includes(decoration.title)}
                <button
                  class="h-7 rounded-md border px-2.5 text-[12px] {active ? 'border-accent bg-accent text-on-accent hover:bg-accent-hover' : 'border-field-line bg-field hover:bg-hover'}"
                  data-annotate="button-decoration"
                  aria-pressed={active}
                  disabled={busy}
                  title={decoration.context}
                  onclick={() => toggleDecoration(decoration.title)}
                >{decoration.title}</button>
              {/each}
            </div>
          {:else}
            <p class="text-[11px] text-muted">Add <code class="font-mono">decorations</code> to the config file to show toggle buttons here.</p>
          {/if}
        </div>

        <label class="flex items-start gap-2">
          <input type="checkbox" bind:checked={greeting} class="mt-0.5" data-annotate="checkbox-greeting" disabled={busy} />
          <span>
            <span class="block text-[12px] font-medium">Greeting and keigo</span>
            <span class="block text-[11px] text-muted">Add an opening and closing line</span>
          </span>
        </label>
      </div>
      <div class="shrink-0 border-t border-line p-3">
        <button
          class="flex h-8 w-full items-center justify-center gap-1.5 rounded-md bg-accent text-[13px] font-medium text-on-accent hover:bg-accent-hover disabled:hover:bg-accent"
          data-annotate="button-generate"
          disabled={!source.trim() || busy || initializing}
          onclick={compose}
        >
          {#if busy}
            <LoaderCircle size={15} class="spin" />Generating…
          {:else}
            <Sparkles size={15} />{replies.length ? 'Regenerate' : 'Generate'}
            <span class="ml-1 text-[11px] opacity-75">{keys.enter}</span>
          {/if}
        </button>
      </div>
    </section>

    <section class="flex min-w-0 flex-1 flex-col" aria-label="Replies" aria-busy={busy}>
      <div class={paneHeader}>
        Replies
        {#if replies.length}<span class="rounded bg-hover px-1.5 font-normal tracking-normal normal-case">{replies.length}</span>{/if}
      </div>
      <div class="min-h-0 flex-1 overflow-auto">
        {#if busy}
          <div class="flex h-full items-center justify-center gap-2 text-muted" role="status">
            <LoaderCircle size={16} class="spin" />Generating {count === 1 ? 'a reply' : `${count} replies`}…
          </div>
        {:else if replies.length}
          <div class="flex flex-col">
            {#each replies as reply, i}
              <ReplyCard {reply} index={i} onerror={(message) => (error = message)} />
            {/each}
          </div>
        {:else}
          <div class="flex h-full items-center justify-center">
            <p class="text-muted">No replies yet. Paste an email and press {keys.enter}.</p>
          </div>
        {/if}
      </div>
    </section>
  </div>

  <footer class="flex h-6 shrink-0 items-center gap-4 border-t border-line bg-toolbar px-3 text-[11px] text-muted">
    <button class="flex items-center gap-1.5 hover:text-fg" data-annotate="button-status-settings" onclick={() => (settings = true)}>
      <span class="h-2 w-2 rounded-full {desktop && config.configured ? 'bg-ok' : 'bg-warn'}"></span>
      {initializing ? 'Loading settings…' : !desktop ? 'Browser preview' : config.configured ? `${config.ai.provider} · ${config.ai.model}` : 'AI provider not configured'}
    </button>
    {#if notice}<span class="truncate">{notice}</span>{/if}
    <span class="ml-auto truncate" title={savedTo ?? undefined}>{savedTo ? `Saved: ${savedTo}` : ''}</span>
  </footer>
</div>

{#if locked}
  <div class="fixed inset-0 z-20 flex items-start justify-center bg-scrim pt-16" role="presentation">
    <div
      class="flex w-[560px] max-w-[calc(100%-32px)] flex-col rounded-lg border border-line bg-window shadow-2xl"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="apikey-title"
      aria-describedby="apikey-desc"
      tabindex="-1"
    >
      <div class="flex gap-3 p-4">
        <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-danger-bg text-danger">
          <KeyRound size={18} />
        </div>
        <div class="flex min-w-0 flex-1 flex-col gap-3">
          <div>
            <h2 id="apikey-title" class="text-[14px] font-semibold">OpenAI API key is not set</h2>
            <p id="apikey-desc" class="mt-1 text-[12px] leading-relaxed text-muted">
              Toneweave can't generate replies without an API key. Add your OpenAI API key to the config file, then click Reload.
            </p>
          </div>
          <div>
            <div class="mb-1 text-[12px] font-medium">Config file</div>
            <code class="selectable block rounded border border-line bg-pane px-2 py-1.5 font-mono text-[12px] break-all">{config.config_path}</code>
          </div>
          <div>
            <div class="mb-1 text-[12px] font-medium">Add this</div>
            <pre class="selectable rounded border border-line bg-pane px-2 py-1.5 font-mono text-[12px] leading-relaxed">ai:
  provider: openai
  api_key: "sk-..."</pre>
          </div>
          <p class="text-[11px] leading-relaxed text-muted">
            See <code class="font-mono">config.example.yaml</code> for all options. You can also set <code class="font-mono">OPENAI_API_KEY</code>, but apps opened from Finder don't read your shell environment.
          </p>
          {#if error}<p class="selectable text-[12px] text-danger" role="alert">{error}</p>{/if}
        </div>
      </div>
      <div class="flex justify-end border-t border-line px-4 py-3">
        <button
          class="flex h-7 items-center gap-1.5 rounded-md bg-accent px-3 text-[12px] font-medium text-on-accent hover:bg-accent-hover"
          data-annotate="button-reload-apikey"
          disabled={initializing}
          onclick={() => reload()}
        >
          {#if initializing}<LoaderCircle size={14} class="spin" />{:else}<RefreshCw size={14} />{/if}Reload
        </button>
      </div>
    </div>
  </div>
{:else if settings}
  <div class="fixed inset-0 z-10 flex items-start justify-center bg-scrim pt-16" role="presentation">
    <div
      class="flex w-[520px] max-w-[calc(100%-32px)] flex-col rounded-lg border border-line bg-window shadow-2xl"
      role="dialog"
      aria-modal="true"
      aria-labelledby="settings-title"
      tabindex="-1"
    >
      <div class="flex h-10 items-center border-b border-line pr-2 pl-4">
        <h2 id="settings-title" class="flex-1 text-[13px] font-semibold">Settings</h2>
        <button class="flex h-7 w-7 items-center justify-center rounded hover:bg-hover" data-annotate="button-close-settings" aria-label="Close settings" onclick={() => (settings = false)}>
          <X size={16} />
        </button>
      </div>
      <div class="flex flex-col gap-3 p-4">
        <div>
          <div class="mb-1 text-[12px] font-medium">Config file</div>
          <code class="selectable block rounded border border-line bg-pane px-2 py-1.5 font-mono text-[12px] break-all">{config.config_path}</code>
        </div>
        <table class="w-full text-[12px]">
          <tbody class="[&_td]:border-b [&_td]:border-line [&_td]:py-1.5 [&_td:first-child]:w-[45%] [&_td:first-child]:text-muted">
            <tr><td>Provider</td><td>{config.ai.provider}</td></tr>
            <tr><td>Model</td><td>{config.ai.model}</td></tr>
            <tr><td>API key</td><td class={config.configured ? 'text-ok' : 'text-danger'}>{config.configured ? 'Configured' : 'Not configured'}</td></tr>
            <tr><td>Default reply count</td><td>{config.general.default_n}</td></tr>
            <tr><td>Auto-paste on launch</td><td>{config.general.clipboard_auto_paste ? 'On' : 'Off'}</td></tr>
            <tr><td>Draft autosave</td><td class="selectable break-all">{config.config_path.replace(/config\.yaml$/, 'draft.json')}</td></tr>
            <tr><td>Auto-copy first reply</td><td>{config.general.auto_copy_result ? 'On' : 'Off'}</td></tr>
          </tbody>
        </table>
        <p class="text-[12px] leading-relaxed text-muted">
          Edit the config file, then reload. See <code class="font-mono">config.example.yaml</code>. Set <code class="font-mono">ai.api_key</code> or
          <code class="font-mono">OPENAI_API_KEY</code>. The source email is sent to the AI provider. The current draft, including the source email, is autosaved locally until you clear it with New.
        </p>
      </div>
      <div class="flex justify-end gap-2 border-t border-line px-4 py-3">
        <button class="h-7 rounded-md border border-field-line bg-field px-3 text-[12px] hover:bg-hover" data-annotate="button-settings-done" onclick={() => (settings = false)}>Done</button>
        <button
          class="flex h-7 items-center gap-1.5 rounded-md bg-accent px-3 text-[12px] font-medium text-on-accent hover:bg-accent-hover"
          data-annotate="button-reload-settings"
          disabled={initializing || busy}
          onclick={() => reload()}
        >
          {#if initializing}<LoaderCircle size={14} class="spin" />{:else}<RefreshCw size={14} />{/if}Reload
        </button>
      </div>
    </div>
  </div>
{/if}
