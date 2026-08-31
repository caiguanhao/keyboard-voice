#!/usr/bin/env bash
set -euo pipefail

command -v espeak-ng >/dev/null 2>&1 || {
  echo "Install espeak-ng before generating audio resources." >&2
  exit 1
}

voice="${KEYBOARD_VOICE:-en-us+f3}"

mkdir -p assets/audio

declare -a ids=(
  escape tab backspace enter space insert delete home end page_up page_down
  copy cut paste colon comma backslash slash exclamationmark open_bracket close_bracket
  at hash dollar percent caret ampersand asterisk open_paren close_paren underscore tilde less_than greater_than
  open_curly_bracket close_curly_bracket pipe questionmark semicolon quote backtick minus period plus equals
  shift control alt meta
  arrow_up arrow_down arrow_left arrow_right browser_back
)

for id in "${ids[@]}"; do
  phrase="${id//_/ }"
  case "$id" in
    escape) phrase="Escape" ;; page_up) phrase="Page up" ;; page_down) phrase="Page down" ;;
    backspace) phrase="Backspace" ;; open_bracket) phrase="Open bracket" ;;
    close_bracket) phrase="Close bracket" ;; open_curly_bracket) phrase="Open curly bracket" ;;
    close_curly_bracket) phrase="Close curly bracket" ;; questionmark) phrase="Question mark" ;;
    at) phrase="At sign" ;; hash) phrase="Hash" ;; dollar) phrase="Dollar sign" ;;
    percent) phrase="Percent" ;; caret) phrase="Caret" ;; ampersand) phrase="Ampersand" ;;
    asterisk) phrase="Asterisk" ;; open_paren) phrase="Left parenthesis" ;;
    close_paren) phrase="Right parenthesis" ;; underscore) phrase="Underscore" ;;
    tilde) phrase="Tilde" ;; less_than) phrase="Less than" ;; greater_than) phrase="Greater than" ;;
    exclamationmark) phrase="Exclamation mark" ;; plus) phrase="Plus" ;; equals) phrase="Equals" ;;
    browser_back) phrase="Back" ;; meta) phrase="Command" ;;
  esac
  espeak-ng -v "$voice" -s 150 -p 52 -a 165 -w "assets/audio/${id}.wav" "$phrase"
done

for n in {0..9}; do
  names=(Zero One Two Three Four Five Six Seven Eight Nine)
  espeak-ng -v "$voice" -s 150 -p 52 -a 165 -w "assets/audio/num${n}.wav" "${names[$n]}"
done

for letter in {a..z}; do
  upper="${letter^^}"
  espeak-ng -v "$voice" -s 150 -p 52 -a 165 -w "assets/audio/${letter}.wav" "$upper"
done

for n in {1..35}; do
  espeak-ng -v "$voice" -s 150 -p 52 -a 165 -w "assets/audio/f${n}.wav" "F $n"
done

echo "Generated complete audio resources in assets/audio/."
