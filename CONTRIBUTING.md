# Contributing to Video Downloader

> 📖 **Languages:** [English](./CONTRIBUTING.md) | [日本語](./japanese/CONTRIBUTING_ja.md)

Thank you for your interest in contributing to Video Downloader! We welcome contributions from everyone.

## How to Contribute

### Reporting Bugs

If you find a bug, please create an issue with:
- A clear and descriptive title
- Steps to reproduce the problem
- Expected behavior
- Actual behavior
- Your environment (OS, Rust version, yt-dlp version)
- Any relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are welcome! Please create an issue with:
- A clear and descriptive title
- Detailed explanation of the proposed feature
- Why this enhancement would be useful
- Examples of how it would work

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following our code guidelines
3. **Test your changes** thoroughly
4. **Commit your changes** with clear, descriptive commit messages
5. **Push to your fork** and submit a pull request

#### Pull Request Guidelines

- Keep changes focused - one feature or fix per PR
- Update documentation for any changed functionality
- Add tests for new features
- Ensure all tests pass
- Follow the existing code style
- Write clear commit messages

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd Downloader

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo test

# Run the application
cargo run
```

### Code Style

This project follows standard Rust conventions:
- Use `cargo fmt` to format your code
- Use `cargo clippy` to check for common mistakes
- Write clear, self-documenting code
- Add comments for complex logic
- Follow Rust naming conventions

### Testing

- Add unit tests for new functionality
- Ensure all existing tests pass with `cargo test`
- Test manually with different platforms (YouTube, Twitch, Twitter/X)
- Test edge cases and error conditions

## Code of Conduct

This project adheres to a [Code of Conduct](./Code_of_Conduct.md). By participating, you are expected to uphold this code.

## Questions?

Feel free to create an issue for any questions about contributing!

## License

By contributing, you agree that your contributions will be licensed under the BSD-2-Clause License.
