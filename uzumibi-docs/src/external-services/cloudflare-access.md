# Cloudflare Access identity

`Uzumibi::Access` retrieves the identity associated with a user authenticated by [Cloudflare Access](https://developers.cloudflare.com/cloudflare-one/access-controls/applications/http-apps/authorization-cookie/). It is available only in Cloudflare Workers projects generated with the `enable-external` feature:

~~~bash
uzumibi new --template cloudflare --features enable-external my-app
~~~

This API is for user sessions authenticated through the `CF_Authorization` cookie. It does not validate Access JWTs locally. Instead, it sends that cookie to Cloudflare's Access identity endpoint, which returns the full identity payload. See Cloudflare's [application-token documentation](https://developers.cloudflare.com/cloudflare-one/access-controls/applications/http-apps/authorization-cookie/application-token/) for the underlying endpoint and payload.

## Configure the team name

Set the Access team name once while the application is loaded. Use the team subdomain only: for `https://my-team.cloudflareaccess.com`, set `"my-team"`.

~~~ruby
class App < Uzumibi::Router
  Uzumibi::Access.team = "my-team"

  # routes ...
end
~~~

`team=` is process-wide configuration for the Wasm instance; do not set it from individual requests.

## Retrieve the current user

Read the `CF_Authorization` cookie from the incoming request and pass it to `get_identity`.

~~~ruby
get "/me" do |req, res|
  token = req.cookie["CF_Authorization"]

  if token.nil? || token.empty?
    res.return(
      401,
      { "content-type" => "application/json" },
      JSON.generate({ "error" => "authentication required" })
    )
  else
    begin
      identity = Uzumibi::Access.get_identity(token)

      res.return(
        200,
        { "content-type" => "application/json" },
        JSON.generate({
          "id" => identity.user_uuid,
          "email" => identity.email
        })
      )
    rescue => error
      debug_console("Cloudflare Access identity lookup failed: #{error.message}")
      res.return(
        401,
        { "content-type" => "application/json" },
        JSON.generate({ "error" => "invalid or expired Access session" })
      )
    end
  end
end
~~~

The example assumes that the route is protected by an Access application. Cloudflare normally checks the `CF_Authorization` cookie before forwarding a protected request; the explicit missing-cookie check also makes the route behave predictably in local development and when its Access policy changes.

## `Uzumibi::AccessIdentity`

`get_identity` returns an `Uzumibi::AccessIdentity` object:

| Property | Meaning |
| --- | --- |
| `user_uuid` | Cloudflare Access user identifier |
| `email` | Authenticated user's email address |
| `raw_data` | Complete identity payload, parsed into Ruby data |

Cloudflare may include more fields in its identity payload than Uzumibi exposes as convenience accessors. Use `raw_data` when you need those fields, and avoid returning it directly to clients because it can contain identity-provider and device information.

## Scope and error handling

- `get_identity` makes an outbound request, so it requires the `enable-external` build and its Asyncify toolchain.
- The current API accepts an Access user-session cookie. It does not accept `CF-Access-Client-Id` / `CF-Access-Client-Secret` service-token credentials.
- A missing, invalid, or expired token is not converted into a special Uzumibi result object. Treat exceptions from `get_identity` as authentication failure, as in the example above.
- Only `user_uuid` and `email` are copied to dedicated accessors. Read additional claims from `raw_data` deliberately and validate their presence before using them for authorization.
