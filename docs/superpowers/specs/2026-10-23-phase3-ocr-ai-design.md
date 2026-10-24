# Phase 3 — OCR Snap + AI (design)

The user asked for Phase 3 to proceed autonomously ("if you need input just decide yourself"),
so every choice below is the builder's call, recorded here so it can be reviewed.

## Intent
Grab text from anything on screen (a video frame, an image, an error dialog, a terminal in a
screenshot) with one hotkey, and get it on the clipboard. Optionally ask an AI to read it
when Windows' OCR isn't good enough, and ask quick questions from the bar.

## Decisions
| Area | Decision | Why |
|---|---|---|
| Hotkey | `tauri-plugin-global-shortcut`, default **Win+Shift+T**, changeable in Settings → OCR | official plugin; spec default |
| Capture | Screenshot the monitor under the cursor (GDI), show it in a fullscreen `snap` window, drag a rectangle, Esc cancels | freezing the frame means video text can't move while selecting |
| OCR engine (default) | **Windows.Media.Ocr**, user-profile languages | built into Windows, offline, no install |
| Tesseract | **Not included** | needs a 30 MB+ install plus language data; Windows OCR covers the same ground. Add it if Windows OCR proves weak for a language the user needs |
| AI vision | **Claude** (Messages API, image input) as the optional engine; key stored in **Windows Credential Manager** (`keyring` crate), never in config.json | one provider keeps it small; GPT can be added behind the same `engine` setting later |
| Modes | **Text** (lines joined), **Code** (indentation rebuilt from word positions), **Terminal** (strip prompts such as `$ `, `PS C:\…> `, `> `) | spec |
| After capture | auto-copy (setting), toast with the first line, saved to history | spec |
| History | SQLite `ocr_history` (last 200), list with copy/delete in the OCR widget panel | spec |
| AI Usage widget | quick-ask box (Claude); shows today's / this month's **token counts taken from the API's own `usage` field**; no cost estimate (prices change; never fabricate) | spec, plus the honesty rule |

## Non-goals
- Live OCR of video without freezing a frame.
- Translating text (it can be done via quick-ask).
- Sending anything to an AI unless the user picked the AI engine or typed a question.

## Limitations to document
- Windows OCR quality depends on the installed language packs (Settings › Time & language).
- The AI engine needs the user's own Anthropic API key and internet; screenshots are sent to Anthropic only in that mode.
- Win+key hotkeys reserved by Windows can't be used; GlassBar reports when registering fails.
