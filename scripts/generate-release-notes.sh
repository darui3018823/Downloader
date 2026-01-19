#!/bin/bash
# generate-release-notes.sh
# Conventional Commits に基づいてリリースノートを生成

PREVIOUS_TAG=$1
CURRENT_TAG=$2

if [ -z "$PREVIOUS_TAG" ] || [ -z "$CURRENT_TAG" ]; then
    echo "Usage: $0 <previous-tag> <current-tag>"
    exit 1
fi

# コミットのリスト取得
COMMITS=$(git log ${PREVIOUS_TAG}..${CURRENT_TAG} --pretty=format:"%h %s")

# カテゴリごとに分類
FEATURES=""
FIXES=""
DOCS=""
REFACTOR=""
CHORE=""
PERF=""
OTHER=""

while IFS= read -r line; do
    if [ -z "$line" ]; then
        continue
    fi
    
    HASH=$(echo "$line" | awk '{print $1}')
    MESSAGE=$(echo "$line" | cut -d' ' -f2-)
    
    if [[ $MESSAGE =~ ^feat(\(.+\))?:\ ]]; then
        FEATURES+="- ${MESSAGE#feat*: } (${HASH})"$'\n'
    elif [[ $MESSAGE =~ ^fix(\(.+\))?:\ ]]; then
        FIXES+="- ${MESSAGE#fix*: } (${HASH})"$'\n'
    elif [[ $MESSAGE =~ ^docs(\(.+\))?:\ ]]; then
        DOCS+="- ${MESSAGE#docs*: } (${HASH})"$'\n'
    elif [[ $MESSAGE =~ ^refactor(\(.+\))?:\ ]]; then
        REFACTOR+="- ${MESSAGE#refactor*: } (${HASH})"$'\n'
    elif [[ $MESSAGE =~ ^perf(\(.+\))?:\ ]]; then
        PERF+="- ${MESSAGE#perf*: } (${HASH})"$'\n'
    elif [[ $MESSAGE =~ ^chore(\(.+\))?:\ ]]; then
        CHORE+="- ${MESSAGE#chore*: } (${HASH})"$'\n'
    else
        OTHER+="- $MESSAGE (${HASH})"$'\n'
    fi
done <<< "$COMMITS"

# リリースノートを出力
echo "# Release ${CURRENT_TAG}"
echo ""

if [ ! -z "$FEATURES" ]; then
    echo "## ✨ Features"
    echo -e "$FEATURES"
    echo ""
fi

if [ ! -z "$FIXES" ]; then
    echo "## 🐛 Bug Fixes"
    echo -e "$FIXES"
    echo ""
fi

if [ ! -z "$PERF" ]; then
    echo "## ⚡ Performance"
    echo -e "$PERF"
    echo ""
fi

if [ ! -z "$REFACTOR" ]; then
    echo "## 🔧 Refactoring"
    echo -e "$REFACTOR"
    echo ""
fi

if [ ! -z "$DOCS" ]; then
    echo "## 📚 Documentation"
    echo -e "$DOCS"
    echo ""
fi

if [ ! -z "$CHORE" ]; then
    echo "## 🧹 Chores"
    echo -e "$CHORE"
    echo ""
fi

if [ ! -z "$OTHER" ]; then
    echo "## Other Changes"
    echo -e "$OTHER"
    echo ""
fi
