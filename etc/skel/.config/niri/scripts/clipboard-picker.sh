#!/bin/bash
# Clipboard history picker with file/image preview support
# Uses cliphist + wofi with Occult Umbral theme
# Super+V opens picker, Enter auto-pastes into focused area

TMPDIR="${XDG_RUNTIME_DIR:-/tmp}/cliphist-previews"
mkdir -p "$TMPDIR"

cleanup() {
    rm -rf "$TMPDIR"
}
trap cleanup EXIT

# Get clipboard history
ITEMS=$(cliphist list)
[ -z "$ITEMS" ] && exit 0

# Enhance entries with type indicators
ENHANCED=""
while IFS= read -r item; do
    TMPFILE="$TMPDIR/clipitem_$(echo "$item" | md5sum | cut -d' ' -f1)"
    echo "$item" | cliphist decode > "$TMPFILE" 2>/dev/null

    MIME=$(file --mime-type -b "$TMPFILE" 2>/dev/null)

    case "$MIME" in
        image/*)
            DIMS=$(identify -format "%wx%h" "$TMPFILE" 2>/dev/null || echo "")
            ENHANCED+="[IMG${DIMS:+ ${DIMS}}] ${item}"$'\n'
            ;;
        video/*)
            ENHANCED+="[VIDEO] ${item}"$'\n'
            ;;
        audio/*)
            ENHANCED+="[AUDIO] ${item}"$'\n'
            ;;
        application/pdf)
            FSIZE=$(du -h "$TMPFILE" 2>/dev/null | cut -f1)
            ENHANCED+="[PDF ${FSIZE}] ${item}"$'\n'
            ;;
        text/*|application/json|application/xml)
            PREVIEW=$(head -c 80 "$TMPFILE" 2>/dev/null | tr '\n' ' ')
            ENHANCED+="${item}  ─ ${PREVIEW}"$'\n'
            ;;
        application/x-shellscript|application/x-executable|inode/x-empty)
            PREVIEW=$(head -c 60 "$TMPFILE" 2>/dev/null | tr '\n' ' ')
            ENHANCED+="${item}  ─ ${PREVIEW}"$'\n'
            ;;
        *)
            CONTENT=$(cat "$TMPFILE" 2>/dev/null)
            if [ -f "$CONTENT" ]; then
                FNAME=$(basename "$CONTENT")
                FSIZE=$(du -h "$CONTENT" 2>/dev/null | cut -f1)
                FMIME=$(file --mime-type -b "$CONTENT" 2>/dev/null)
                case "$FMIME" in
                    image/*) ENHANCED+="[FILE:img] ${FNAME} (${FSIZE}) | ${item}"$'\n' ;;
                    video/*) ENHANCED+="[FILE:vid] ${FNAME} (${FSIZE}) | ${item}"$'\n' ;;
                    audio/*) ENHANCED+="[FILE:aud] ${FNAME} (${FSIZE}) | ${item}"$'\n' ;;
                    *)       ENHANCED+="[FILE] ${FNAME} (${FSIZE}) | ${item}"$'\n' ;;
                esac
            else
                PREVIEW=$(head -c 60 "$TMPFILE" 2>/dev/null | tr '\n' ' ')
                ENHANCED+="${item}  ─ ${PREVIEW}"$'\n'
            fi
            ;;
    esac
done <<< "$ITEMS"

ENHANCED=$(echo "$ENHANCED" | sed '/^[[:space:]]*$/d')
[ -z "$ENHANCED" ] && exit 0

# Show in wofi
SELECTED=$(echo "$ENHANCED" | wofi \
    --show dmenu \
    --prompt "Clipboard History" \
    --width 600 \
    --height 400 \
    --style "${XDG_CONFIG_HOME:-$HOME/.config}/wofi/style.css" \
    2>/dev/null)

[ -z "$SELECTED" ] && exit 0

# Extract the original cliphist key
if echo "$SELECTED" | grep -q '^\['; then
    ORIGINAL_KEY=$(echo "$SELECTED" | sed 's/^\[[^]]*\] //')
else
    ORIGINAL_KEY=$(echo "$SELECTED" | sed 's/  ─ .*//')
fi

# Decode and copy to clipboard directly (avoid echo corruption of binary data)
printf '%s' "$ORIGINAL_KEY" | cliphist decode | wl-copy --type text/plain

# Auto-paste: -s sleeps AFTER connecting to Wayland but BEFORE sending keys,
# giving wofi time to fully close and focus to return to the previous window.
# Ctrl+Shift+V works in terminals (alacritty, etc.) - standard paste shortcut.
wtype -s 800 -M ctrl -M shift v
