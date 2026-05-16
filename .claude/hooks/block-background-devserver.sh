#!/bin/bash
# PreToolUse hook: block background dev server launches inside Claude Code.
# Claude Code sends tool JSON to stdin; exit 2 to block with a message.
# See .claude/rules/claude-code-background-shell.md for the full rationale.

INPUT=$(cat)

RESULT=$(echo "$INPUT" | python3 -c "
import sys, json, re

try:
    data = json.load(sys.stdin)
    ti = data.get('tool_input', data)
    bg = ti.get('run_in_background', False)
    cmd = ti.get('command', '')

    dev_pattern = re.compile(
        r'(pnpm|npm|yarn|bun)\s+(run\s+)?dev\b'
        r'|cargo\s+tauri\s+dev\b'
        r'|next\s+dev\b'
        r'|trunk\s+serve\b'
        r'|\bvite\b'
        r'|\bturbopack\b'
    )

    if bg and dev_pattern.search(cmd):
        print('BLOCK')
    else:
        print('OK')
except Exception:
    print('OK')  # fail open on parse errors
")

if [ "$RESULT" = "BLOCK" ]; then
    echo "BLOCKED: Do not start a dev server in the background inside Claude Code."
    echo "Use 'cargo tauri dev' in a separate terminal window instead."
    echo "Reason: Claude Code captures all stdout/stderr of background processes"
    echo "into /private/tmp/claude-*/tasks/ with no size limit or cleanup."
    echo "Reference: .claude/rules/claude-code-background-shell.md"
    exit 2
fi

exit 0
