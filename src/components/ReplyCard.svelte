<script lang="ts">
  import { Check, Copy } from 'lucide-svelte';
  import { copy, replyText, type Reply } from '../lib/api';
  let { reply, index, onerror }: { reply: Reply; index: number; onerror: (message: string) => void } = $props();
  let copied = $state(false);
  async function copyReply() {
    try { await copy(replyText(reply)); copied = true; setTimeout(() => copied = false, 2200); }
    catch { onerror('Could not copy this reply. Please try again.'); }
  }
</script>
<article class="reply-card">
  <div class="reply-card-top"><span class="variant-number">0{index + 1}</span><span class="tone-tag">{reply.tone_used}</span><button class="icon-button" aria-label={`Copy variant ${index + 1}`} onclick={copyReply}>{#if copied}<Check size={16} />{:else}<Copy size={16} />{/if}</button></div>
  <h3>{reply.subject_line}</h3>
  <p class="reply-body">{reply.reply_text}</p>
  <button class="copy-link" onclick={copyReply}>{#if copied}<Check size={13} /> Copied{:else}<Copy size={13} /> Copy reply{/if}</button>
</article>
