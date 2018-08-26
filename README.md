# Lua Async HTTP
This is a work-in-progress implementation of a cross-platform async HTTP library for Lua 5.1. I'm hoping to use it as part of an integration test suite for [Rojo](https://github.com/LPGhatguy/rojo) or even shipping it as part of [Lemur](https://github.com/LPGhatguy/lemur).

## Why?
LuaSocket's HTTP interface is synchronous. The API I'm trying to emulate, Roblox's `HttpService`, is implemented via coroutines. Since LuaSocket's APIs are blocking and Lua is single-threaded, testing Rojo accurately isn't really possible. During normal plugin operation, Rojo is allowed to do other things while a request is outstanding, but with blocking requests, it can't.

Other options would be pulling another library from the Lua ecosystem, but they generally have poor Windows support or are otherwise difficult to build. This project builds with just `cargo build` on every platform.

## Usage
The library's API is **horrible** right now. It's based on numeric status codes and polling for completion -- this is because I want Lua to be in control of task scheduling, unlike other Lua solutions.

In general:

```lua
local socket = require("socket")
local async_http = require("async_http")

local handle = async_http.request("https://google.com/")

while true do
	local success, status, result = async_http.check_request(handle)

	if not success then
		error("Handle was invalid, this shouldn't happen.")
	end

	print("Checking request status...")

	if status == 0 then
		print("Request is still in flight.")
		socket.sleep(0.2)
	elseif status == 1 then
		print("Success! Body: ", result)
		break
	elseif status == 2 then
		error("Error! Message: " .. result)
	else
		error("Unexpected status code: " .. status)
	end
end
```

## TODO
I stopped working on this library in favor of using Roblox itself for integration tests when possible. I'll revisit this eventually when I figure out whether the project would be integrated into Lemur or Rojo, and what the API should actually be.

* [ ] Move `success` value to `request` instead of `check_request`
* [ ] Optionally supply event loop?
* [ ] Use strings or userdata instead of numeric status codes
* [ ] Return table for response with:
	* `body`
	* `status_code`
	* `headers`

## License
Available under the MIT license. See [LICENSE.txt](LICENSE.txt) for details.