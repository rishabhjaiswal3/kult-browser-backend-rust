#!/bin/bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:4100}"

login_user() {
  local wallet="$1"
  local name="$2"
  local payload resp

  payload=$(jq -nc --arg w "$wallet" --arg n "$name" '{walletAddress:$w,name:$n}')
  resp=$(curl -sS -X POST "$BASE_URL/player/login" \
    -H 'Content-Type: application/json' \
    --data "$payload")

  if [[ "$(echo "$resp" | jq -r '.ok')" != "true" ]]; then
    echo "login failed for $name: $resp" >&2
    exit 1
  fi

  echo "$resp"
}

create_moment() {
  local token="$1"
  local title="$2"
  local description="$3"
  local tags_json="$4"
  local url="$5"
  local payload resp

  payload=$(jq -nc \
    --arg title "$title" \
    --arg description "$description" \
    --arg url "$url" \
    --argjson tags "$tags_json" \
    '{title:$title,description:$description,tags:$tags,assetUrl:$url}')

  resp=$(curl -sS -X POST "$BASE_URL/moments/register" \
    -H "Authorization: Bearer $token" \
    -H 'Content-Type: application/json' \
    --data "$payload")

  if [[ "$(echo "$resp" | jq -r '.ok')" != "true" ]]; then
    echo "create moment failed: $resp" >&2
    exit 1
  fi

  echo "$resp"
}

create_comment() {
  local token="$1"
  local moment_id="$2"
  local content="$3"
  local payload resp

  payload=$(jq -nc --arg content "$content" '{content:$content}')
  resp=$(curl -sS -X POST "$BASE_URL/moments/$moment_id/comments" \
    -H "Authorization: Bearer $token" \
    -H 'Content-Type: application/json' \
    --data "$payload")

  if [[ "$(echo "$resp" | jq -r '.ok')" != "true" ]]; then
    echo "create comment failed: $resp" >&2
    exit 1
  fi

  echo "$resp"
}

create_reply() {
  local token="$1"
  local comment_id="$2"
  local content="$3"
  local payload resp

  payload=$(jq -nc --arg content "$content" '{content:$content}')
  resp=$(curl -sS -X POST "$BASE_URL/moments/comments/$comment_id/replies" \
    -H "Authorization: Bearer $token" \
    -H 'Content-Type: application/json' \
    --data "$payload")

  if [[ "$(echo "$resp" | jq -r '.ok')" != "true" ]]; then
    echo "create reply failed: $resp" >&2
    exit 1
  fi

  echo "$resp"
}

owner_name_for() {
  case "$1" in
    alice) echo "Alice Moments" ;;
    bob) echo "Bob Comments" ;;
    cara) echo "Cara Replies" ;;
  esac
}

token_for() {
  case "$1" in
    alice) echo "$ALICE_TOKEN" ;;
    bob) echo "$BOB_TOKEN" ;;
    cara) echo "$CARA_TOKEN" ;;
  esac
}

commenter_for() {
  case "$1" in
    alice) echo "bob" ;;
    bob) echo "cara" ;;
    cara) echo "alice" ;;
  esac
}

extra_for() {
  case "$1" in
    alice) echo "cara" ;;
    bob) echo "alice" ;;
    cara) echo "bob" ;;
  esac
}

alice=$(login_user "0xa11ce11111111111111111111111111111111111" "Alice Moments")
bob=$(login_user "0xb0b2222222222222222222222222222222222222" "Bob Comments")
cara=$(login_user "0xcaa3333333333333333333333333333333333333" "Cara Replies")

ALICE_TOKEN=$(echo "$alice" | jq -r '.data.token')
BOB_TOKEN=$(echo "$bob" | jq -r '.data.token')
CARA_TOKEN=$(echo "$cara" | jq -r '.data.token')

URLS=(
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/1774452872330-1d389928-dcf3-488f-8ffe-0f76c8394731-zero-dash-desktop-2.png"
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/68747470733a2f2f796176757a63656c696b65722e6769746875622e696f2f73616d706c652d696d616765732f696d6167652d313032312e6a7067.jpeg"
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/6c81N9uua8m4SKKAIERuW.jpg"
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/cploxaXqymrdky9PXvP0d.jpeg"
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/crypto-crypto-cosmos.gif"
  "https://kult-browser.sfo3.digitaloceanspaces.com/moments/download.jpeg"
)

OWNERS=(alice alice bob bob cara cara)

TITLES=(
  "Zero Dash Desktop Sample"
  "Hex Image Sample"
  "Skyline Sample Shot"
  "Warm Tone Sample"
  "Cosmos Motion Sample"
  "Downloaded Sample"
)

DESCRIPTIONS=(
  "Seeded sample moment for comments and replies testing."
  "Seeded sample moment using provided media asset."
  "Testing the public feed with seeded moments."
  "Testing comments and reply counters on sample media."
  "Animated sample used for moment flow checks."
  "Last seeded sample to round out the test set."
)

TAGS=(
  "[\"sample\",\"dashboard\",\"image\"]"
  "[\"sample\",\"hex\",\"image\"]"
  "[\"sample\",\"jpg\",\"feed\"]"
  "[\"sample\",\"warm\",\"jpeg\"]"
  "[\"sample\",\"gif\",\"motion\"]"
  "[\"sample\",\"download\",\"image\"]"
)

FIRST_COMMENT_ID=""

for i in "${!URLS[@]}"; do
  owner="${OWNERS[$i]}"
  owner_token=$(token_for "$owner")
  commenter=$(commenter_for "$owner")
  commenter_token=$(token_for "$commenter")
  extra=$(extra_for "$owner")
  extra_token=$(token_for "$extra")

  moment_resp=$(create_moment "$owner_token" "${TITLES[$i]}" "${DESCRIPTIONS[$i]}" "${TAGS[$i]}" "${URLS[$i]}")
  moment_id=$(echo "$moment_resp" | jq -r '.data.momentId')
  printf 'created moment %s by %s\n' "$moment_id" "$(owner_name_for "$owner")"

  comment_resp=$(create_comment "$commenter_token" "$moment_id" "Nice sample. Testing comments on ${TITLES[$i]}.")
  comment_id=$(echo "$comment_resp" | jq -r '.data.commentId')
  if [[ -z "$FIRST_COMMENT_ID" ]]; then
    FIRST_COMMENT_ID="$comment_id"
  fi
  printf '  comment %s by %s\n' "$comment_id" "$(owner_name_for "$commenter")"

  reply_resp=$(create_reply "$owner_token" "$comment_id" "Replying as the moment owner on ${TITLES[$i]}.")
  reply_id=$(echo "$reply_resp" | jq -r '.data.commentId')
  printf '  reply %s by %s\n' "$reply_id" "$(owner_name_for "$owner")"

  if [[ "$i" -lt 3 ]]; then
    extra_resp=$(create_comment "$extra_token" "$moment_id" "Second top-level comment for ${TITLES[$i]} from another tester.")
    extra_comment_id=$(echo "$extra_resp" | jq -r '.data.commentId')
    printf '  extra comment %s by %s\n' "$extra_comment_id" "$(owner_name_for "$extra")"
  fi
done

feed=$(curl -sS "$BASE_URL/moments?page=1&perPage=10")
if [[ "$(echo "$feed" | jq -r '.ok')" != "true" ]]; then
  echo "feed fetch failed: $feed" >&2
  exit 1
fi

printf 'seeded feed count on page: %s\n' "$(echo "$feed" | jq -r '.data.moments | length')"
printf 'newest moment numComments: %s\n' "$(echo "$feed" | jq -r '.data.moments[0].numComments')"

if [[ -n "$FIRST_COMMENT_ID" ]]; then
  replies=$(curl -sS "$BASE_URL/moments/comments/$FIRST_COMMENT_ID/replies?page=1&perPage=10")
  if [[ "$(echo "$replies" | jq -r '.ok')" == "true" ]]; then
    printf 'first thread replies returned: %s\n' "$(echo "$replies" | jq -r '.data.comments | length')"
  fi
fi
