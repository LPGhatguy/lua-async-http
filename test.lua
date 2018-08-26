-- In order to use the library directly from the build directory on 32-bit windows:
-- package.cpath = package.cpath .. ";target/i686-pc-windows-msvc/debug/?.dll"

local socket = require("socket")
local async_http = require("async_http")

local handle = async_http.request("https://google.com/")

while true do
	local success, status, result = async_http.check_request(handle)

	if not success then
		error("failed!")
	end

	print("checking...")

	if status == 0 then
		print("in flight")
		socket.sleep(2)
	elseif status == 1 then
		print("body: " .. result)
		break
	elseif status == 2 then
		error("error: " .. result)
	else
		error("unknown status: " .. status)
	end
end