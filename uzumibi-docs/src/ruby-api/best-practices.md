# Best Practices

1. **Always return `res`**: Make sure to return the response object from every route handler
2. **Set Content-Type**: Always set appropriate `Content-Type` header
3. **Use appropriate status codes**: Return correct HTTP status codes for different scenarios
4. **Validate input**: Check and validate request parameters and body
5. **Keep routes simple**: Complex logic should be extracted into helper methods or classes
6. **Use debug_console carefully**: Excessive logging can impact performance
