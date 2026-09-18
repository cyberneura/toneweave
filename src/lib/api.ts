import { invoke, isTauri } from '@tauri-apps/api/core';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
export interface Reply { reply_text: string; subject_line: string; tone_used: string }
export interface Generation { variants: Reply[]; saved_to: string | null; copied: boolean; warnings: string[] }
export interface Config {
  ai: { provider: string; model: string; base_url: string };
  general: { clipboard_auto_paste: boolean; auto_copy_result: boolean; default_n: number };
  greeting: { enabled: boolean; opening: string; closing: string };
  preset_prompts: { name: string; prompt: string }[];
  configured: boolean; config_path: string;
}
export const desktop = isTauri();
export const defaults: Config = {
  ai: { provider: 'openai', model: 'gpt-4.1-mini', base_url: 'https://api.openai.com/v1' },
  general: { clipboard_auto_paste: false, auto_copy_result: false, default_n: 3 },
  greeting: { enabled: true, opening: '', closing: '' },
  preset_prompts: [
    { name: '丁寧に', prompt: 'Thoughtful and professional' }, { name: 'カジュアルに', prompt: 'Warm and conversational' },
    { name: '簡潔に', prompt: 'Short and to the point' }, { name: '熱意を伝える', prompt: 'Positive and enthusiastic' },
  ], configured: false, config_path: '~/.config/toneweave/config.yaml',
};
export const getConfig = () => desktop ? invoke<Config>('get_config') : Promise.resolve(defaults);
export async function generate(source: string, direction: string, preset: string, greeting: boolean) {
  if (!desktop) throw new Error('Open the Toneweave desktop app to generate replies. Run pnpm tauri dev.');
  return invoke<Generation>('generate_reply', { source, direction, preset, greeting });
}
export const paste = () => desktop ? readText() : navigator.clipboard.readText();
export const copy = (text: string) => desktop ? invoke<void>('copy_to_clipboard', { text }) : navigator.clipboard.writeText(text);
export const replyText = (reply: Reply) => `Subject: ${reply.subject_line}\n\n${reply.reply_text}`;
