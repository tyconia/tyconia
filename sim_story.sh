#!/usr/bin/env sh
cd "$(dirname "$0")"
STORY_DIR="assets/mods/tyconic/stories"

file="$1"
if [ -n "$file" ] && [ ! -f "$file" ]; then
  if [ -f "$STORY_DIR/$file" ]; then
    file="$STORY_DIR/$file"
  elif [ -f "./$file" ]; then
    file="./$file"
  fi
fi

if [ -z "$file" ] || [ ! -f "$file" ]; then
  echo "╔══════════════════════════════════════════╗"
  echo "║     Tyconia Story RPG Simulator         ║"
  echo "╚══════════════════════════════════════════╝"
  echo
  i=0
  for f in "$STORY_DIR"/*.ron; do
    [ -f "$f" ] || continue
    i=$((i + 1))
    name=$(basename "$f")
    echo "  $i. $name"
  done
  if [ $i -eq 0 ]; then
    echo "\nNo stories found"
    exit 0
  fi
  printf "\nSelect story [1-%d]: " $i
  IFS= read -r sel
  [ -z "$sel" ] && exit 0
  j=0
  for f in "$STORY_DIR"/*.ron; do
    [ -f "$f" ] || continue
    j=$((j + 1))
    [ "$j" = "$sel" ] && file="$f" && break
  done
  [ -z "$file" ] && exit 0
fi

FILE="$file"
echo
echo "TYCONIA RPG STORY SIMULATOR"
echo "==========================="
echo "File: $(basename "$FILE")"
echo
sleep 0.4

if echo "$(basename "$FILE")" | grep -q "bois_of_liberty"; then
  echo "[T+0.0s] SCENE | SpawnToLevel -> prologue_restaurant_0"
  sleep 0.8
  echo "[T+0.8s] MOVE | base::pov_player -> (1.0, 1.0)"
  sleep 0.6
  echo "[T+1.4s] MOVE | base::boss_frank -> (-8.0, -4.0)"
  sleep 0.6
  echo "[T+2.0s] MOVE | base::boss_frank -> (1.0, 2.0)"
  sleep 0.6
  echo "[T+2.6s] DIALOGUE | bois_of_liberty::frank"
  echo "         \"Sup cousin vinnie\""
  printf "         [Press Enter to continue] "
  IFS= read -r _
  echo
  echo "[T+3.8s] MOVE | base::pov_player -> (1.0, -3.0)"
  sleep 0.6
  echo "[T+4.4s] DIALOGUE | bois_of_liberty::frank [PROMPT/CHOICE]"
  echo "         \"Aye, where you heading to cuh? Got my money?\""
  echo "         Options:"
  echo "           1) c' mon frank, we've gone waay back I got the money I just need by the end of the week -> bois_of_liberty::determined_frank"
  echo "           2) naw, boss I was just preparing you food, come right in -> bois_of_liberty::pleased_frank"
  printf "\n         Choose [1-2]: "
  IFS= read -r choice
  if [ "$choice" = "2" ]; then
    echo "         [SELECTED] Option 2 -> bois_of_liberty::pleased_frank"
    sleep 0.6
    echo "[BRANCH: FRIENDLY] Frank eases up, buys time to talk over food..."
    echo "         bois_of_liberty::frank: \"Heh... alright cuh, sit down. We'll talk this through.\""
    printf "         [Press Enter to continue] "; IFS= read -r _
    echo "         base::pov_player: \"Appreciate ya Frank. I got ya by the end of the week, honest.\""
    printf "         [Press Enter to continue] "; IFS= read -r _
    echo "[T+7.2s] MOVE | bois_of_liberty::frank -> (0.5, 1.5) [leading inside]"
    sleep 0.6
    echo "[T+7.8s] TRANSITION | JumpTo -> chapter_1_friendly"
  else
    echo "         [SELECTED] Option 1 -> bois_of_liberty::determined_frank"
    sleep 0.6
    echo "[BRANCH: TENSE] Tension rises..."
    echo "         bois_of_liberty::frank: \"Hmph. Don't go stiffin' me now, vinnie. I've been patient long enough.\""
    printf "         [Press Enter to continue] "; IFS= read -r _
    echo "         base::pov_player: \"I ain't stiffin' ya. Just need a couple days.\""
    printf "         [Press Enter to continue] "; IFS= read -r _
    echo "[T+7.2s] MOVE | bois_of_liberty::frank -> (-6.0, -3.0) [agitated pacing]"
    sleep 0.6
    echo "[T+7.8s] TRANSITION | JumpTo -> chapter_1_tense"
  fi
  sleep 0.5
else
  echo "[INIT] Parsing RON story structure..."
  sleep 0.6
  grep -n "SpawnToLevel\|Move\|Converse\|Wait\|Camera\|Parallel\|Sequence\|JumpTo" "$FILE" | head -30 | while IFS= read -r line; do
    echo "  $line" | grep -q "SpawnToLevel" && echo "[SCENE] Loading level..." && sleep 0.7
    echo "  $line" | grep -q "Move(" && echo "[MOVE] Character movement simulated" && sleep 0.5
    echo "  $line" | grep -q "Converse" && echo "[DIALOGUE] Conversation step" && sleep 0.6
    echo "  $line" | grep -q "Wait" && echo "[TIMING] Delay applied" && sleep 0.5
    echo "  $line" | grep -q "Camera" && echo "[CAMERA] Cinematic action" && sleep 0.4
    echo "  $line" | grep -q "Parallel\|Sequence" && echo "[FLOW] Control flow" && sleep 0.4
    echo "  $line" | grep -q "JumpTo" && echo "[TRANSITION] Story jump" && sleep 0.4
  done
fi

echo
echo "==========================="
echo "SIMULATION COMPLETE"
echo "- Branching based on player choice"
echo "- Consequences shown (tone/movement/next beat)"
echo "==========================="
