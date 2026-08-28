local M = {}

local defaults = {
	binary_path = nil,
	signs = {
		enable = true,
		icon = "⚡",
		priority = 10,
	},
	virtual_text = {
		enable = true,
		prefix = " ⚡ Voltage: ",
	},
	functions = {
		enable = true,
		min_complexity = 5,
		show_name = true,
	},
	thresholds = {
		high = 50,
		medium = 20,
		low = 5,
	},
	auto_annotate = true,
	border = "rounded",
	picker = "auto", -- "auto" | "snacks" | "telescope"
}

M.config = vim.deepcopy(defaults)
M.cache = {
	results = {},
	by_path = {},
}

local volt_ns = vim.api.nvim_create_namespace("VoltHighVoltage")

local function get_score_hl(score)
	local t = M.config.thresholds or defaults.thresholds
	if score >= t.high then
		return "DiagnosticError"
	elseif score >= t.medium then
		return "DiagnosticWarn"
	elseif score >= t.low then
		return "DiagnosticInfo"
	else
		return "DiagnosticHint"
	end
end

local function get_binary_path()
	if M.config.binary_path and vim.fn.executable(M.config.binary_path) == 1 then
		return M.config.binary_path
	end

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

function M.annotate_buffer(bufnr)
	bufnr = bufnr or vim.api.nvim_get_current_buf()
	if not vim.api.nvim_buf_is_valid(bufnr) or not vim.api.nvim_buf_is_loaded(bufnr) then
		return
	end

	local buf_name = vim.api.nvim_buf_get_name(bufnr)
	if buf_name == "" then
		return
	end

	local relative_name = vim.fn.fnamemodify(buf_name, ":.")
	local item = M.cache.by_path[relative_name] or M.cache.by_path[buf_name]

	vim.api.nvim_buf_clear_namespace(bufnr, volt_ns, 0, -1)

	if not item then
		return
	end

	local line_count = vim.api.nvim_buf_line_count(bufnr)
	if line_count == 0 then
		return
	end

	-- 1. File-level annotation on line 0
	local file_hl = get_score_hl(item.score)
	local file_opts = {}

	if M.config.signs.enable then
		file_opts.sign_text = M.config.signs.icon or "⚡"
		file_opts.sign_hl_group = file_hl
		file_opts.priority = M.config.signs.priority or 10
	end

	if M.config.virtual_text.enable then
		local prefix = M.config.virtual_text.prefix or " ⚡ Voltage: "
		file_opts.virt_text = {
			{ string.format("%s%.1f (churn: %d, complexity: %d)", prefix, item.score, item.churn or 0, item.complexity or 0), file_hl },
		}
	end

	if next(file_opts) ~= nil then
		pcall(vim.api.nvim_buf_set_extmark, bufnr, volt_ns, 0, 0, file_opts)
	end

	-- 2. Function-level annotations
	local func_cfg = M.config.functions or defaults.functions
	if func_cfg.enable and item.functions and #item.functions > 0 then
		local min_comp = func_cfg.min_complexity or 5

		for _, func in ipairs(item.functions) do
			if func.complexity >= min_comp and func.line and func.line > 0 then
				local target_line = math.min(func.line - 1, line_count - 1)
				-- Don't overwrite line 0 if function is on line 1
				if target_line > 0 then
					local func_hl = get_score_hl(func.score)
					local func_opts = {}

					if M.config.signs.enable then
						func_opts.sign_text = M.config.signs.icon or "⚡"
						func_opts.sign_hl_group = func_hl
						func_opts.priority = (M.config.signs.priority or 10) - 1
					end

					if M.config.virtual_text.enable then
						local name_str = func_cfg.show_name and string.format("fn %s ", func.name) or ""
						func_opts.virt_text = {
							{ string.format(" ⚡ %s(Voltage: %.1f | complexity: %d)", name_str, func.score, func.complexity), func_hl },
						}
					end

					if next(func_opts) ~= nil then
						pcall(vim.api.nvim_buf_set_extmark, bufnr, volt_ns, target_line, 0, func_opts)
					end
				end
			end
		end
	end
end

function M.annotate_all_buffers()
	for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
		if vim.api.nvim_buf_is_loaded(bufnr) then
			M.annotate_buffer(bufnr)
		end
	end
end

function M.scan_project(callback)
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
			if not ok or type(results) ~= "table" then
				if #table.concat(data) > 0 then
					vim.notify("Volt: Failed to parse JSON output.", vim.log.levels.WARN)
				end
				return
			end

			M.cache.results = results
			M.cache.by_path = {}
			for _, item in ipairs(results) do
				if item.file_path then
					M.cache.by_path[item.file_path] = item
				end
			end

			M.annotate_all_buffers()

			if callback then
				callback(results)
			else
				vim.notify(string.format("Volt: Scanned %d hotspot files.", #results), vim.log.levels.INFO)
			end
		end,
		on_stderr = function(_, data)
			local err_msg = table.concat(data, "\n"):gsub("^%s*(.-)%s*$", "%1")
			if #err_msg > 0 then
				vim.notify("Volt Error: " .. err_msg, vim.log.levels.WARN)
			end
		end,
	})
end

function M.show_summary()
	if M.cache.results and #M.cache.results > 0 then
		require("volt.ui").show_summary(M.cache.results, M.config, function()
			M.scan_project(function(results)
				require("volt.ui").show_summary(results, M.config)
			end)
		end)
	else
		M.scan_project(function(results)
			require("volt.ui").show_summary(results, M.config, function()
				M.scan_project(function(new_results)
					require("volt.ui").show_summary(new_results, M.config)
				end)
			end)
		end)
	end
end

local function has_snacks()
	return pcall(require, "snacks") or _G.Snacks ~= nil
end

local function has_telescope()
	return pcall(require, "telescope")
end

function M.picker(picker_type)
	local chosen = picker_type or M.config.picker

	if chosen == "auto" then
		if has_snacks() then
			chosen = "snacks"
		elseif has_telescope() then
			chosen = "telescope"
		else
			M.show_summary()
			return
		end
	end

	local run_picker = function(results)
		if chosen == "snacks" then
			require("volt.snacks").picker(results, M.config)
		elseif chosen == "telescope" then
			require("volt.telescope").picker(results, M.config)
		else
			require("volt.ui").show_summary(results, M.config)
		end
	end

	if M.cache.results and #M.cache.results > 0 then
		run_picker(M.cache.results)
	else
		M.scan_project(function(results)
			run_picker(results)
		end)
	end
end

function M.snacks()
	M.picker("snacks")
end

function M.telescope()
	M.picker("telescope")
end

function M.clear()
	M.cache.results = {}
	M.cache.by_path = {}
	for _, bufnr in ipairs(vim.api.nvim_list_bufs()) do
		if vim.api.nvim_buf_is_valid(bufnr) then
			vim.api.nvim_buf_clear_namespace(bufnr, volt_ns, 0, -1)
		end
	end
	vim.notify("Volt: Cleared all annotations.", vim.log.levels.INFO)
end

function M.setup(user_opts)
	M.config = vim.tbl_deep_extend("force", defaults, user_opts or {})

	if M.config.auto_annotate then
		local augroup = vim.api.nvim_create_augroup("VoltAutoAnnotate", { clear = true })
		vim.api.nvim_create_autocmd({ "BufReadPost", "BufEnter" }, {
			group = augroup,
			callback = function(args)
				M.annotate_buffer(args.buf)
			end,
		})
	end
end

return M
