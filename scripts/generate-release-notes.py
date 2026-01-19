#!/usr/bin/env python3
"""
Generate categorized release notes from Conventional Commits.

Usage: python3 generate-release-notes.py <previous-tag> <current-tag>
"""

import subprocess
import re
import sys
from collections import defaultdict

def get_commits(from_tag, to_tag):
    """Fetch commits between two tags."""
    try:
        result = subprocess.run(
            ["git", "log", f"{from_tag}..{to_tag}", "--pretty=format:%h %s"],
            capture_output=True,
            text=True,
            check=False
        )
        commits = result.stdout.strip().split('\n') if result.stdout.strip() else []
        return commits
    except Exception:
        return []

def categorize_commits(commits):
    """Categorize commits by Conventional Commits type."""
    categories = defaultdict(list)
    
    for commit in commits:
        if not commit.strip():
            continue
            
        parts = commit.split(' ', 1)
        if len(parts) != 2:
            continue
            
        hash_short = parts[0]
        message = parts[1]
        
        # Match conventional commit format
        match = re.match(r'^(feat|fix|docs|refactor|perf|chore|build|ci|style|test|revert)(\(.+?\))?:\s+(.+)$', message)
        
        if match:
            commit_type = match.group(1)
            detail = match.group(3)
            categories[commit_type].append((detail, hash_short))
        else:
            categories['other'].append((message, hash_short))
    
    return categories

def format_release_notes(current_tag, categories):
    """Format categorized commits as release notes."""
    type_emoji = {
        'feat': '✨ Features',
        'fix': '🐛 Bug Fixes',
        'perf': '⚡ Performance',
        'refactor': '🔧 Refactoring',
        'docs': '📚 Documentation',
        'chore': '🧹 Chores',
        'build': '🏗️ Build',
        'ci': '🤖 CI/CD',
        'style': '💄 Style',
        'test': '✅ Tests',
        'revert': '⏮️ Reverts',
        'other': 'Other'
    }
    
    output = [f"# Release {current_tag}", ""]
    
    # Define order
    order = ['feat', 'fix', 'perf', 'refactor', 'docs', 'chore', 'build', 'ci', 'style', 'test', 'revert', 'other']
    
    for commit_type in order:
        if commit_type not in categories or not categories[commit_type]:
            continue
        
        emoji_header = type_emoji.get(commit_type, commit_type)
        output.append(f"## {emoji_header}")
        
        for detail, hash_short in categories[commit_type]:
            output.append(f"- {detail} ({hash_short})")
        
        output.append("")
    
    return '\n'.join(output)

def main():
    if len(sys.argv) < 3:
        print("Usage: generate-release-notes.py <previous-tag> <current-tag>", file=sys.stderr)
        sys.exit(1)
    
    previous_tag = sys.argv[1]
    current_tag = sys.argv[2]
    
    commits = get_commits(previous_tag, current_tag)
    
    if not commits:
        print(f"# Release {current_tag}")
        return
    
    categories = categorize_commits(commits)
    release_notes = format_release_notes(current_tag, categories)
    
    print(release_notes)

if __name__ == '__main__':
    main()
