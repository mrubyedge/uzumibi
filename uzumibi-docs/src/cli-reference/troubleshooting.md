# Troubleshooting

### "uzumibi: command not found"

Make sure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your shell profile (`~/.bashrc`, `~/.zshrc`, etc.) to make it permanent.

### "Invalid template"

Check that you're using a valid template name:
- `cloudflare`
- `fastly`
- `spin`
- `cloudrun`

Template names are case-sensitive and must be lowercase.

### "Directory already exists"

The CLI won't overwrite existing directories. Either:
1. Choose a different project name
2. Remove the existing directory
3. Use a different location
