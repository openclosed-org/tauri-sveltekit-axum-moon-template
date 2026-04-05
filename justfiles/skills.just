# AI Skills Management — unified interface for humans & agents
# Manages AI agent skills via the `skills` CLI tool
# Skills are installed to .agents/skills/ (project-local, committed to git)

# ── Discovery ───────────────────────────────────────────────

# List all installed skills
skills-list:
    @echo "=== Installed AI Skills ==="
    @bunx skills list
    @echo ""
    @echo "💡 Add more skills: just skills-add <github-repo-or-url>"

# Search for available skills
skills-find QUERY='':
    @if [ -z "{{QUERY}}" ]; then \
        bunx skills find; \
    else \
        bunx skills find "{{QUERY}}"; \
    fi

# Check for skill updates
skills-check:
    @echo "=== Checking for Skill Updates ==="
    @bunx skills check

# ── Installation ───────────────────────────────────────────

# Add skills from GitHub/GitLab/local directory
skills-add SOURCE='':
    @test -n "{{SOURCE}}" || (echo "Usage: just skills-add <github-repo-or-url>" && echo "Examples:" && echo "  just skills-add vercel-labs/agent-skills" && echo "  just skills-add https://github.com/owner/repo" && echo "  just skills-add ./path/to/skill" && exit 1)
    @echo "=== Adding Skill: {{SOURCE}} ==="
    @bunx skills add "{{SOURCE}}"

# Add specific skill from a repository (skip others)
skills-add-specific SOURCE='' SKILL='':
    @test -n "{{SOURCE}}" || (echo "Usage: just skills-add-specific <repo> <skill-name>" && exit 1)
    @test -n "{{SKILL}}" || (echo "Usage: just skills-add-specific <repo> <skill-name>" && exit 1)
    @echo "=== Adding Specific Skill: {{SKILL}} from {{SOURCE}} ==="
    @bunx skills add "{{SOURCE}}" -s "{{SKILL}}"

# Update all installed skills
skills-update:
    @echo "=== Updating All Skills ==="
    @bunx skills update

# ── Management ──────────────────────────────────────────────

# Remove a skill
skills-remove SKILL='':
    @test -n "{{SKILL}}" || (echo "Usage: just skills-remove <skill-name>" && echo "Available skills:" && bunx skills list && exit 1)
    @echo "=== Removing Skill: {{SKILL}} ==="
    @bunx skills remove "{{SKILL}}"

# Initialize a new custom skill template
skills-init NAME='':
    @test -n "{{NAME}}" || (echo "Usage: just skills-init <skill-name>" && exit 1)
    @echo "=== Creating Skill: {{NAME}} ==="
    @bunx skills init "{{NAME}}"
    @echo ""
    @echo "💡 Edit the template at: .agents/skills/{{NAME}}/SKILL.md"

# ── Information ─────────────────────────────────────────────

# Show skill status and directory structure
skills-status:
    @echo "=== Skills Directory Structure ==="
    @echo ""
    @bunx skills list

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
