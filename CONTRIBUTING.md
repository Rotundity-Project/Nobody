# Contributing

## 1. Development Setup
```bash
npm install
npm run tauri:dev
```

Backend checks:
```bash
cd src-tauri
cargo test
```

Frontend checks:
```bash
npm run lint
npm run test:unit
```

## 2. Code Style
- Frontend: TypeScript + Vue 3 Composition API
- Backend: Rust with `rustfmt` and `clippy`
- Keep changes focused and minimal per commit
- Add tests for behavior changes

## 3. Branch And Commit
- Use feature branches from `main`
- Follow Conventional Commits
  - `feat:`
  - `fix:`
  - `perf:`
  - `docs:`
  - `chore:`

Example:
```bash
git commit -m "feat(script): add novel import character selection"
```

## 4. Pull Request Checklist
- Build passes locally
- Related tests added/updated
- No unrelated file changes included
- User-facing behavior documented when needed

## 5. Reporting Issues
When opening issues, include:
- environment (OS, Node, Rust versions)
- reproduction steps
- expected vs actual result
- logs or screenshots if available
