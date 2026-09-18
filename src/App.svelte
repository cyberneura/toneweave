<script lang="ts">
  import { onMount } from 'svelte';
  import { ArrowRight, ArrowUpRight, Check, ChevronDown, Clipboard, Feather, FileText, LoaderCircle, Mail, Plus, Settings2, ShieldCheck, Sparkles, WandSparkles, X } from 'lucide-svelte';
  import ReplyCard from './components/ReplyCard.svelte';
  import { defaults, desktop, generate, getConfig, paste, type Config, type Reply } from './lib/api';
  let source = $state('');
  let direction = $state('');
  let preset = $state('丁寧に');
  let greeting = $state(true);
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
  let words = $derived(source.trim() ? source.trim().split(/\s+/).length : 0);
  const suggestions = ['やんわり断る', '日程を調整する', '感謝を伝える'];
  async function reload(initial = false) {
    initializing = true; error = '';
    try {
      config = await getConfig(); greeting = config.greeting.enabled;
      if (!config.preset_prompts.some(p => p.name === preset)) preset = config.preset_prompts[0]?.name ?? '';
      if (initial && config.general.clipboard_auto_paste && !source) await pasteEmail();
    } catch (e) { error = String(e); }
    finally { initializing = false; }
  }
  onMount(() => { void reload(true); });
  async function pasteEmail() {
    try { const text = await paste(); if (text?.trim()) { source = text; notice = 'Email pasted from clipboard.'; } else { error = 'Your clipboard has no text to paste.'; } }
    catch { error = 'Could not read the clipboard. Paste directly into the email field with ⌘V.'; }
  }
  async function compose() {
    if (busy || initializing || !source.trim()) return;
    busy = true; error = ''; notice = '';
    try {
      const result = await generate(source, direction, preset, greeting);
      replies = result.variants; savedTo = result.saved_to;
      notice = result.warnings.length ? result.warnings.join(' ') : result.copied ? 'Replies ready. First variant copied to clipboard.' : `${result.variants.length} ways to say it. Choose the one that feels like you.`;
    } catch (e) { error = String(e); }
    finally { busy = false; }
  }
  function newDraft() {
    if ((source || direction || replies.length) && !confirmNew) { confirmNew = true; return; }
    source = ''; direction = ''; replies = []; error = ''; notice = ''; savedTo = null; confirmNew = false;
    sourceInput?.focus();
  }
  function keyboard(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') { event.preventDefault(); void compose(); }
    if (event.key === 'Escape') { settings = false; confirmNew = false; }
  }
</script>

<svelte:window onkeydown={keyboard} />
<div class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <a class="brand" href="#" aria-label="Toneweave home"><span class="brand-mark"><Feather size={19} strokeWidth={1.7} /></span>Toneweave<span class="beta">DESKTOP</span></a>
    <div class="header-actions"><span class="local-badge"><span></span>Your words, thoughtfully woven</span><button class="icon-button" aria-label="Settings" onclick={() => settings = true}><Settings2 size={18} /></button></div>
  </header>
  <main>
    <section class="intro"><div><div class="eyebrow"><span></span>A LITTLE DIRECTION. THE RIGHT WORDS.</div><h1>Make room for a better reply.</h1><p>Bring the email. Add your intention. Find your voice.</p></div><button class="secondary-button new-draft" disabled={busy} onclick={newDraft}><Plus size={15} />New draft</button></section>
    {#if !desktop}<div class="preview-note"><FileText size={14} /> Workspace preview · Generate replies in the desktop app with <code>pnpm tauri dev</code>.</div>{/if}
    {#if confirmNew}<div class="message-bar" role="alert"><span>Clear this workspace and start a new draft? Saved result files will remain.</span><button onclick={newDraft}>Start fresh</button><button onclick={() => confirmNew = false}>Keep writing</button></div>{/if}
    <div class="workspace">
      <section class="panel source-panel" aria-labelledby="source-title">
        <div class="panel-heading"><div class="step">01</div><div><h2 id="source-title">The original email</h2><p>A little context goes a long way.</p></div><Mail size={19} class="heading-icon" /></div>
        <div class="panel-content source-content"><div class="field-heading"><label for="source">SOURCE EMAIL</label><button class="text-button" disabled={busy} onclick={pasteEmail}><Clipboard size={13} /> Paste</button></div>
          <textarea bind:this={sourceInput} id="source" class="source-text" bind:value={source} disabled={busy} maxlength={100000} placeholder={'Paste the email you’d like to reply to…\n\nYou can include the subject, the sender’s message, and any context that helps.'}></textarea>
          <div class="textarea-footer"><span>{source.length.toLocaleString()} characters</span><span>{words} words</span></div>
          <div class="source-hint"><ShieldCheck size={15} /><span>Just a starting point.<br />Your source email isn’t saved locally.</span></div>
        </div>
      </section>
      <section class="panel direction-panel" aria-labelledby="direction-title">
        <div class="panel-heading"><div class="step">02</div><div><h2 id="direction-title">Your direction</h2><p>You set the intention. We find the words.</p></div><WandSparkles size={19} class="heading-icon" /></div>
        <div class="panel-content direction-content">
          <label class="field-label" for="direction">WHAT WOULD YOU LIKE TO SAY?</label>
          <textarea id="direction" class="direction-text" rows="3" bind:value={direction} disabled={busy} maxlength={8000} placeholder="e.g. やんわり断りつつ、次の機会につなげたい"></textarea>
          <div class="suggestions">{#each suggestions as suggestion}<button disabled={busy} onclick={() => direction = suggestion}>{suggestion}<ArrowUpRight size={11} /></button>{/each}</div>
          <div class="divider"></div>
          <label class="field-label" for="preset">SET THE TONE</label>
          <div class="select-wrap"><Sparkles size={15} /><select id="preset" bind:value={preset} disabled={busy || initializing}>{#each config.preset_prompts as item}<option value={item.name}>{item.name}</option>{/each}</select><ChevronDown size={15} /></div>
          <p class="field-help">A starting tone, with your own touch.</p>
          <div class="greeting-row"><div><label for="greeting">A thoughtful greeting</label><p>敬語 {greeting ? 'ON' : 'OFF'} · Opening & closing</p></div><button id="greeting" class:enabled={greeting} class="switch" role="switch" aria-checked={greeting} aria-label="Greeting and keigo" disabled={busy} onclick={() => greeting = !greeting}><span></span></button></div>
          <div class="intention-note"><span class="note-spark">✳</span><p>You don’t need the perfect prompt.<br />A few honest words are enough.</p></div>
        </div>
        <div class="direction-bottom"><span class="small-dot"></span>{config.general.default_n} perspectives. One intention.</div>
      </section>
      <section class="panel reply-panel" aria-labelledby="reply-title" aria-busy={busy}>
        <div class="panel-heading"><div class="step green-step">03</div><div><h2 id="reply-title">Your reply, reimagined</h2><p>A few possibilities. All yours.</p></div><span class="variant-count">{config.general.default_n} variants</span></div>
        <div class="reply-scroll">
          {#if busy}<div class="empty-state loading-state" role="status"><div class="empty-art"><LoaderCircle size={32} class="spin" /></div><h3>Finding the right words…</h3><p>Weaving your intention into {config.general.default_n} distinct replies.</p><div class="loading-lines"><span></span><span></span><span></span></div></div>
          {:else if replies.length}<div class="reply-list">{#each replies as reply, i}<ReplyCard {reply} index={i} onerror={(message) => error = message} />{/each}</div>
          {:else}<div class="empty-state"><div class="empty-art"><div class="paper-lines"><span></span><span></span><span></span></div><Feather size={29} strokeWidth={1.4} /></div><span class="empty-eyebrow">A BLANK PAGE, FULL OF POSSIBILITY</span><h3>The right words are on their way.</h3><p>Add an email and a little direction.<br />Your reply takes shape here.</p><div class="empty-flow"><Mail size={15} /><span>·····</span><Sparkles size={16} /><span>·····</span><FileText size={15} /></div></div>{/if}
        </div>
        <div class="generate-area"><button class="generate-button" onclick={compose} disabled={!source.trim() || busy || initializing}>{#if busy}<LoaderCircle size={17} class="spin" />Weaving replies…{:else}<Sparkles size={17} />{replies.length ? 'Regenerate replies' : 'Generate replies'}<span class="key-hint">⌘ ↵</span>{/if}</button><p>Drafted with AI. Always reviewed by you.</p></div>
      </section>
    </div>
    {#if error}<div class="error-banner" role="alert"><span>{error}</span><button class="icon-button" aria-label="Dismiss error" onclick={() => error = ''}><X size={15}/></button></div>{/if}
    {#if notice}<div class="notice" role="status"><Check size={14} />{notice}</div>{/if}
    <footer><span><ShieldCheck size={13} /> A workspace for drafts. You decide what gets sent.</span><span title={savedTo ?? undefined}>{#if savedTo}<Check size={12} /> Saved to results/{:else}Made for the way you mean it.{/if}</span></footer>
  </main>
  <div class="statusbar"><button onclick={() => settings = true}><span class:ready={desktop && config.configured} class="status-dot"></span>{initializing ? 'Loading settings…' : !desktop ? 'Browser preview' : config.configured ? config.ai.model : 'Set up your AI provider'}<ChevronDown size={12} /></button><span>TONEWEAVE <span class="status-separator">/</span> A calmer way to reply <Feather size={12} /></span></div>
</div>
{#if settings}
  <div class="modal-backdrop" role="presentation"><section class="settings-dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title" tabindex="-1">
    <div class="settings-heading"><div><span class="eyebrow">MAKE IT YOURS</span><h2 id="settings-title">Workspace settings</h2></div><button class="icon-button" aria-label="Close settings" onclick={() => settings = false}><X size={19} /></button></div>
    <p>Configure your provider, presets and clipboard preferences in:</p><code class="config-path">{config.config_path}</code>
    <dl><div><dt>Provider</dt><dd>{config.ai.provider}</dd></div><div><dt>Model</dt><dd>{config.ai.model}</dd></div><div><dt>API key</dt><dd>{config.configured ? 'Configured' : 'Not configured'}</dd></div><div><dt>Variants per generation</dt><dd>{config.general.default_n}</dd></div><div><dt>Auto-paste / auto-copy</dt><dd>{config.general.clipboard_auto_paste ? 'On' : 'Off'} / {config.general.auto_copy_result ? 'On' : 'Off'}</dd></div></dl>
    <p class="settings-help">Start with <code>config.example.yaml</code> in the project. Set <code>ai.api_key</code> or <code>OPENAI_API_KEY</code>. Reload after editing. Source emails are sent to your configured AI provider when you generate; only generated replies are saved locally.</p>
    <button class="generate-button" disabled={initializing || busy} onclick={() => reload()}>{#if initializing}<LoaderCircle size={16} class="spin" />{:else}<Settings2 size={16} />{/if}Reload settings<ArrowRight size={15} /></button>
    {#if error}<p class="settings-error" role="alert">{error}</p>{/if}
  </section></div>
{/if}
