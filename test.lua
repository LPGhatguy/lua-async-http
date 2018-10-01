-- In order to use the library directly from the build directory on 32-bit Windows:
package.cpath = package.cpath .. ";target/i686-pc-windows-msvc/debug/?.dll"

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