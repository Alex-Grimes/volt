local M = {}

local function get_binary_path()
	local script_path = debug.getinfo(1, "S").source:sub(2)
	local plugin_root = vim.fn.fnamemodify(script_path, ":h:h:h")

	local candidates = {
		plugin_root .. "/target/release/volt-core", -- Release build (fastest)
		plugin_root .. "/target/debug/volt-core", -- Debug build (dev default)
		plugin_root .. "/bin/volt-core", -- Custom bin folder (optional)
		"volt-core", -- System PATH (fallback)
	}

	for _, path in ipairs(candidates) do
		if vim.fn.executable(path) == 1 then
			return path
		end
	end

	return nil
end

local volt_ns = vim.api.nvim_create_namespace("VoltHighVoltage")

function M.scan_project()
	local bin = get_binary_path()

	if not bin then
		vim.notify("Volt: Could not find 'volt-core' binary. Did you run 'cargo build'?", vim.log.levels.ERROR)
		return
	end

	vim.fn.jobstart({ bin }, {
		stdout_buffered = true,
		on_stdout = function(_, data)
			if not data or #data == 0 then
				return
			end

			local ok, results = pcall(vim.json.decode, table.concat(data))
			if not ok then
				if #table.concat(data) > 0 then
					vim.notify("Volt: Failed to parse JSON output.", vim.log.levels.WARN)
				end
				return
			end

			vim.api.nvim_buf_clear_namespace(0, volt_ns, 0, -1)

			for _, item in ipairs(results) do
				local path = item.file_path

				if path then
					local bufnr = vim.fn.bufnr(path)

					if bufnr ~= -1 then
						vim.api.nvim_buf_set_extmark(bufnr, volt_ns, 0, 0, {
							sign_text = "⚡",
							sign_hl_group = "DiagnosticWarn",
							virt_text = { { string.format(" Voltage: %.2f", item.score), "Comment" } },
						})
					end
				end
			end
		end,
	})
end

return M
