# Lua Async HTTP
This is a work-in-progress implementation of a cross-platform async HTTP library for Lua 5.1. I'm hoping to use it as part of an integration test suite for [Rojo](https://github.com/LPGhatguy/rojo) or even shipping it as part of [Lemur](https://github.com/LPGhatguy/lemur).

## Why?
LuaSocket's HTTP interface is synchronous. The API I'm trying to emulate, Roblox's `HttpService`, is implemented via coroutines. Since LuaSocket's APIs are blocking and Lua is single-threaded, testing Rojo accurately isn't really possible. During normal plugin operation, Rojo is allowed to do other things while a request is outstanding, but with blocking requests, it can't.

Other options would be pulling another library from the Lua ecosystem, but they generally have poor Windows support or are otherwise difficult to build. This project builds with just `cargo build` on every platform.

## Usage
The library's API is not excellent right now but is functional. It's based on string status codes and polling for completion -- this is because I want Lua to be in control of task scheduling, unlike other Lua solutions.

In general, usage is:

```lua
local async_http = require("async_http")

local initialSuccess, handle = async_http.request("http://example.com")
assert(initialSuccess, handle)

while true do
	local status, result = async_http.check_request(handle)

	print("checking with status ", status)

	if status == "in-flight" then
		async_http.sleep_ms(200)
	elseif status == "success" then
		print("body:")
		print(result)
		break
	elseif status == "error" then
		error("error: " .. result)
	else
		error("unknown status: " .. status)
	end
end

async_http.cleanup_request(handle)
```

## TODO
I stopped working on this library in favor of using Roblox itself for integration tests when possible. I'll revisit this eventually when I figure out whether the project would be integrated into Lemur or Rojo, and what the API should actually be.

* [x] Move `success` value to `request` instead of `check_request`
* [ ] Optionally supply event loop?
* [x] Use strings or userdata instead of numeric status codes
* [ ] Return table for response with:
	* `body`
	* `status_code`
	* `headers`

## License
Available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.