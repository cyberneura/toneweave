<script lang="ts">
  import { Check, Copy } from 'lucide-svelte';
  import { copy, replyText, type Reply } from '../lib/api';
  let { reply, index, onerror }: { reply: Reply; index: number; onerror: (message: string) => void } = $props();
  let copied = $state(false);
  async function copyReply() {
    try {
      await copy(replyText(reply));
      copied = true;
      setTimeout(() => (copied = false), 2000);
    } catch {
      onerror('Could not copy this reply. Please try again.');
    }
  }
</script>

<article class="border-b border-line">
  <div class="flex h-8 items-center gap-2 bg-pane px-3">
    <span class="text-[12px] font-semibold tabular-nums">#{index + 1}</span>
    <span class="rounded border border-line bg-window px-1.5 text-[11px] text-muted">{reply.tone_used}</span>
    <button
      class="ml-auto flex h-6 items-center gap-1 rounded border border-field-line bg-field px-2 text-[12px] hover:bg-hover"
      data-annotate="button-copy-reply"
      onclick={copyReply}
    >
      {#if copied}<Check size={13} class="text-ok" />Copied{:else}<Copy size={13} />Copy{/if}
    </button>
  </div>
  <div class="selectable px-3 py-2.5">
    <h3 class="mb-2 text-[13px] font-semibold">{reply.subject_line}</h3>
    <p class="text-[13px] leading-relaxed break-words whitespace-pre-wrap">{reply.reply_text}</p>
  </div>
</article>
