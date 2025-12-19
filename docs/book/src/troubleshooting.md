# Troubleshooting & Limitations

- **Rule delimiter**: All rules must use `::=`. Using `=` will fail in the macro.
- **Quantifiers**: `[]` (optional) and `{}` (zero-or-more) replace legacy `? * +`. For one-or-more, write `item { item }`.
- **Ranges**: Use `'a'..'z'` instead of `[a-z]`.
- **Streaming window**: Backtracking is limited to the current buffer window; extremely deep backtracking over huge inputs may fail. Consider smaller grammars or AST mode for bounded inputs.
- **Unicode**: Grammars operate on Rust `char`. Ensure your input is valid UTF-8 if using `parse_str`.
- **Error context**: Always inspect `ParseEvent::Error` for hints and rule context. Spans are best-effort when streaming.
