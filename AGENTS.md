# Senior Rust Developer - Clean Code Agent Guidelines

## Core Principles

### 1. Code Quality Standards

- **Readability First**: Code should be self-documenting and easy to understand
- **Simplicity**: Favor simple solutions over complex ones
- **Consistency**: Follow established patterns and conventions throughout the codebase
- **Testability**: Write code that is easy to test and maintain

### 2. Rust-Specific Clean Code Practices

#### Naming Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for types, structs, and enums
- Use `SCREAMING_SNAKE_CASE` for constants
- Choose descriptive names that reveal intent
- Avoid abbreviations unless widely understood

#### Function Design

- Keep functions small and focused (single responsibility)
- Prefer explicit returns over implicit ones for clarity
- Use meaningful parameter names
- Limit function parameters to 3-4 when possible
- Use result types (`Result<T, E>`) for error handling

#### Struct and Organization

- Group related fields together in structs
- Use `#[derive(Debug, Clone, PartialEq)]` judiciously
- Implement `Drop` only when necessary
- Prefer composition over inheritance

#### File and Module Organization

- **One struct/enum per file**: Each struct or enum should be in its own file to maintain clear separation of concerns
- **Module structure**: Create a `mod.rs` in each module directory with module declarations and `pub use` re-exports for a clean API
- **Naming conventions**: Avoid redundant prefixes in filenames when the module name already provides context (e.g., `area_config.rs` → `config.rs` in
  `config/area/` module)
- **Clear separation**: Ensure modules have distinct responsibilities - if types belong to different domains, separate them into different modules

### 3. Error Handling

- Use `Result<T, E>` for recoverable errors
- Use `Option<T>` for values that may be absent
- Create custom error types with `thiserror` or `anyhow`
- Handle errors at the appropriate level
- Avoid using `panic!()` in production code

### 4. Memory Management

- Let Rust's ownership system manage memory
- Use references (`&`) when you don't need ownership
- Consider `Cow<str>` for string handling when appropriate
- Be mindful of lifetimes and avoid unnecessary allocations

### 5. Concurrency

- Use channels for message passing
- Prefer `Arc<Mutex<T>` over `RwLock<T>` when write operations are rare
- Use async/await for I/O-bound operations
- Avoid blocking operations in async contexts

## Code Review Checklist

### Before Submitting Code

- [ ] Code follows Rust naming conventions
- [ ] Functions are small and focused
- [ ] Error handling is comprehensive
- [ ] No unused dependencies or imports
- [ ] Tests cover critical paths
- [ ] Documentation is clear and concise
- [ ] Performance considerations are addressed
- [ ] Security implications are considered

### During Code Review

- [ ] Code is readable and maintainable
- [ ] Abstractions are appropriate
- [ ] No code duplication
- [ ] Proper use of Rust features (iterators, patterns, etc.)
- [ ] Memory usage is efficient
- [ ] Error messages are helpful

## Testing Guidelines

### Unit Tests

- Test public API behavior
- Use descriptive test names
- Test both success and failure cases
- Use `#[should_panic]` for expected panics
- Mock external dependencies when necessary

### Integration Tests

- Test component interactions
- Use realistic test data
- Test error propagation
- Verify performance characteristics

### Documentation Tests

- Include examples in doc comments
- Test code examples with `cargo test --doc`
- Ensure examples compile and run

## Performance Considerations

### Optimization Guidelines

- Profile before optimizing
- Use `#[inline]` judiciously
- Consider `Box<T>` for large types
- Avoid unnecessary allocations

### Memory Efficiency

- Use stack allocation when possible
- Use `String::from` vs `to_string` appropriately
- Be mindful of string allocations in loops

## Tooling and Workflow

### Development Tools

- Use `rustfmt` for consistent formatting
- Use `clippy` for linting and suggestions
- Use `cargo-audit` for security checks
- Use `cargo-deny` for dependency checking

### Git Workflow

- Write clear, descriptive commit messages
- Use conventional commit format
- Keep commits small and focused
- Review own code before requesting review

## Learning and Growth

### Code Quality Metrics

- Monitor cyclomatic complexity
- Track test coverage
- Measure performance regressions
- Review security vulnerabilities

## Project-Specific Requirements (smearor-wrot)

### Rust Implementation Standards

- **Rust Edition 2024**: Use latest edition features
- **Modern Versions**: Keep dependencies updated to modern versions
- **Idiomatic Rust**: Follow Rust best practices and patterns
- **Panic-Free Code**: Avoid `unwrap()`, `expect()`, and panicking code
- **English Comments**: All source code comments in English
- **No Abbreviations**: Use descriptive variable names without abbreviations
- **Prefer Trait Implementations over Free Functions**: Avoid standalone free functions (e.g. `parse_*`, `derive_*`, `serialize_*`) when a standard trait
  implementation such as `FromStr`, `Serialize`, `Deserialize`, or `Display` can encapsulate the same logic. This keeps parsing, serialization, and conversion
  logic co-located with the type definition and ensures a consistent API across the codebase.

### Documentation Standards

- **Type Documentation**: All public enums and structs must have rustdoc comments describing their purpose
- **Enum Variants**: Each enum variant must be documented with a brief description of its meaning
- **Struct Fields**: Each field in a struct must be documented with a description of its purpose and any relevant details
- **Documentation Format**: Use `///` for rustdoc comments, follow rustdoc conventions
- **Language**: All documentation must be in English
- **Clarity**: Documentation should be clear, concise, and focus on semantic meaning

### Import Organization

- **Individual Imports**: One import per line
- **No Star Imports**: Except for preludes (e.g., `gtk4::prelude::*`)
- **No Import Grouping**: Keep imports separate and ungrouped
- **No Import Comments**: Don't comment import statements
- **Macro Usage**: Use `debug!` instead of `tracing::debug!` with proper imports
- **Alphabetical Ordering**: Sort all `use` statements alphabetically within each scope
- **Import Scope Order**: Group by source — `crate::` first, then external crates, then `std::` last

### Formatting

- **Use `rustfmt`**: Run `cargo fmt` before committing to ensure consistent formatting
- **Single-line preference**: Prefer single-line style for macro invocations and function signatures when they remain readable (e.g.
  `impl_json_convertible!(Name, Type, |json| { ... });`)
- **No trailing commas** in single-line constructs; use trailing commas only in multi-line blocks
- **Compact closures**: Keep closure bodies compact when they fit on one line

### Dependencies

- `thiserror`: Internal error types
- `miette`: User-facing error types
- `smithay`: Wayland compositor framework
- `clap`: Command line argument parsing
- `gtk4`: GTK4 framework for UI widgets
- `glib`: GLib utilities and patterns

### Testing Requirements

- **Idiomatic Tests**: Use idiomatic Rust testing patterns
- **Inline Tests**: Keep tests in the same file as the source code
- **Comprehensive Coverage**: Test both success and error paths

### Performance Considerations

- **Hardware Acceleration**: Consider DMA-BUF support for GPU rendering
- **Memory Efficiency**: Optimize for Wayland compositor requirements
- **Input Handling**: Efficient mouse and touch event processing

## Resources

### Documentation

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [GTK4 Rust Documentation](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- [Smithay Documentation](https://smithay.github.io/smithay/)

### Tools and Libraries

- `clippy` - Rust linter
- `rustfmt` - Code formatter
- `cargo-audit` - Security audit
- `thiserror` - Error handling
- `miette` - User-friendly error reporting
- `anyhow` - Error handling
- `tokio` - Async runtime
- `serde` - Serialization
- `smithay` - Wayland compositor framework
- `gtk4` - GTK4 bindings
- `clap` - CLI argument parsing

---

*This guide should be updated regularly to reflect best practices and team experience.*