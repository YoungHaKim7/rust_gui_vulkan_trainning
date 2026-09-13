```bash
git show --stat b5e61c4 | head -8; git status --short | grep -v "^??" | head -3; cargo test --lib 2>&1 | grep "test result"
```
