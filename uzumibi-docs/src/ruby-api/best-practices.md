# Best Practices

- Set `status_code`, `headers`, and `body` on every successful route, or use `res.return`.
- Use `req.raw_body` when you need the original bytes rather than adapter-assisted parsing.
- Check the type of `req.body` before treating it as parsed JSON.
- Keep platform service calls behind small application methods so platform dependencies stay visible.
- Restart the generated development command after changing embedded Ruby code.
- Keep request-size configuration proportional to the platform memory available.
- Test generated projects with the same feature overlay used in deployment.
- Treat provider limits and Wrangler configuration as external, versioned dependencies.
