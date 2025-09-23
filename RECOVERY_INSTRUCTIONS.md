# 🚨 Repository Recovery Instructions

## Quick Recovery (for everyone on the team)

### Step 1: Check what you have locally

Open Git Bash or your terminal and run:
```bash
git status
```

### Step 2: Based on what you see, follow ONE of these paths:

---

## Path A: "Your branch is behind" + NO local changes
**Simplest case - just run:**
```bash
git fetch origin
git reset --hard origin/main
```
✅ Done! You can continue working.

---

## Path B: You have UNCOMMITTED changes (red files in git status)
**Save your work first:**
```bash
# Option 1: Stash them temporarily
git stash

# Then sync with main
git fetch origin
git reset --hard origin/main

# Get your changes back
git stash pop
```

**OR if you prefer to keep them as a commit:**
```bash
# Commit your changes
git add .
git commit -m "WIP: my work"

# Create backup and sync
git branch backup-my-work
git fetch origin
git reset --hard origin/main

# Cherry-pick your work back
git cherry-pick backup-my-work
```

---

## Path C: You have COMMITTED but UNPUSHED work
**Save your commits first:**
```bash
# Create a backup branch with your commits
git branch backup-$(date +%Y%m%d)

# Sync with main
git fetch origin
git reset --hard origin/main

# See what commits you had
git log backup-$(date +%Y%m%d) --oneline

# Cherry-pick the ones you want to keep
git cherry-pick <commit-hash>
```

---

## 🆘 If Something Goes Wrong

1. **DON'T PANIC** - Your work is not lost
2. **DON'T force push anything**
3. Run this to see all your recent work:
   ```bash
   git reflog
   ```
4. Contact the team lead with the output

---

## ✅ How to Verify You're Synced

After recovery, you should see:
```bash
git log --oneline -n 3
```
Should show:
- Latest commit from today
- Version 0.15.2 in the logs
- No mention of .workflow_state files

```bash
grep "^version" Cargo.toml | head -1
```
Should show: `version = "0.15.2"`

---

## 📝 What Happened?

- Sensitive workflow state files were accidentally committed
- Git history was cleaned to remove them
- All work was preserved and recovered
- Version bumped to 0.15.2 with all features intact