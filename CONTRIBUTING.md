# Contributing to tpt-diskview

Thank you for your interest in contributing! Here's how to get started.

## Development Setup

1. Install prerequisites: Rust, Node.js (LTS), pnpm, Tauri CLI
2. Clone the repo
3. Run `pnpm install`
4. Run `pnpm tauri dev` to start in development mode

## Code Style

### Rust

- Format with `cargo fmt`
- Lint with `cargo clippy -- -D warnings`
- Follow standard Rust conventions

### TypeScript/SolidJS

- Format with Prettier
- Lint with ESLint
- Use TypeScript strict mode

## Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make your changes
4. Ensure all checks pass (`cargo fmt --check`, `cargo clippy`, `pnpm lint`)
5. Submit a pull request with a clear description

## Reporting Issues

Use GitHub Issues. Include:
- OS and version
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if applicable

## License

By contributing, you agree that your contributions will be licensed under the same dual license as the project (MIT OR Apache-2.0).
