# AI Skills Management — unified interface for humans & agents
# Manages AI agent skills via the `skills` CLI tool
# Skills are installed to .agents/skills/ (project-local, committed to git)

# ── Discovery ───────────────────────────────────────────────

# List all installed skills
skills-list:
    cargo run -p repo-tools -- dev skills list

# Search for available skills
skills-find QUERY='':
    @if [ -z "{{QUERY}}" ]; then \
        cargo run -p repo-tools -- dev skills find; \
    else \
        cargo run -p repo-tools -- dev skills find "{{QUERY}}"; \
    fi

# Check for skill updates
skills-check:
    cargo run -p repo-tools -- dev skills check

# ── Installation ───────────────────────────────────────────

# Add skills from GitHub/GitLab/local directory
skills-add SOURCE='':
    @test -n "{{SOURCE}}" || (echo "Usage: just skills-add <github-repo-or-url>" && echo "Examples:" && echo "  just skills-add vercel-labs/agent-skills" && echo "  just skills-add https://github.com/owner/repo" && echo "  just skills-add ./path/to/skill" && exit 1)
    cargo run -p repo-tools -- dev skills add "{{SOURCE}}"

# Add specific skill from a repository (skip others)
skills-add-specific SOURCE='' SKILL='':
    @test -n "{{SOURCE}}" || (echo "Usage: just skills-add-specific <repo> <skill-name>" && exit 1)
    @test -n "{{SKILL}}" || (echo "Usage: just skills-add-specific <repo> <skill-name>" && exit 1)
    cargo run -p repo-tools -- dev skills add-specific "{{SOURCE}}" "{{SKILL}}"

# Update all installed skills
skills-update:
    cargo run -p repo-tools -- dev skills update

# ── Management ──────────────────────────────────────────────

# Remove a skill
skills-remove SKILL='':
    @test -n "{{SKILL}}" || (echo "Usage: just skills-remove <skill-name>" && echo "Available skills:" && bunx skills list && exit 1)
    cargo run -p repo-tools -- dev skills remove "{{SKILL}}"

# Initialize a new custom skill template
skills-init NAME='':
    @test -n "{{NAME}}" || (echo "Usage: just skills-init <skill-name>" && exit 1)
    cargo run -p repo-tools -- dev skills init "{{NAME}}"

# ── Information ─────────────────────────────────────────────

# Show skill status and directory structure
skills-status:
    cargo run -p repo-tools -- dev skills status

# Show documentation
skills-help:
    @echo "=== AI Skills Management ==="
    @echo ""
    @echo "Available commands:"
    @echo "  just skills-list              - List installed skills"
    @echo "  just skills-find [query]      - Search available skills"
    @echo "  just skills-add <source>      - Add skill from GitHub/GitLab/local"
    @echo "  just skills-add-specific ...  - Add specific skill from repo"
    @echo "  just skills-update            - Update all installed skills"
    @echo "  just skills-check             - Check for updates"
    @echo "  just skills-remove <name>     - Remove a skill"
    @echo "  just skills-init <name>       - Create new skill template"
    @echo "  just skills-status            - Show directory structure"
    @echo ""
    @echo "Examples:"
    @echo "  just skills-add vercel-labs/agent-skills"
    @echo "  just skills-add https://github.com/owner/repo"
    @echo "  just skills-init release-notes"
    @echo ""
    @echo "💡 See .agents/SKILLS.md for full documentation"
