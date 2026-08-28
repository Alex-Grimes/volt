local M = {}

local volt_ui_ns = vim.api.nvim_create_namespace("VoltUiHighlights")

local function get_score_hl(score, thresholds)
	thresholds = thresholds or { high = 50, medium = 20, low = 5 }
	if score >= thresholds.high then
		return "DiagnosticError"
	elseif score >= thresholds.medium then
		return "DiagnosticWarn"
	elseif score >= thresholds.low then
		return "DiagnosticInfo"
	else
		return "DiagnosticHint"
	end
end

function M.show_summary(results, opts, on_refresh)
	opts = opts or {}
	local thresholds = opts.thresholds or { high = 50, medium = 20, low = 5 }

	if not results or #results == 0 then
		vim.notify("Volt: No high voltage files found.", vim.log.levels.INFO)
		return
	end

	local buf = vim.api.nvim_create_buf(false, true)
	vim.bo[buf].bufhidden = "wipe"
	vim.bo[buf].buftype = "nofile"
	vim.bo[buf].swapfile = false

	local header_title = " ⚡ Volt: High Voltage Hotspot Report ⚡"
	local separator = " " .. string.rep("─", 74)
	local col_header = string.format(" %-40s │ %10s │ %6s │ %10s", "File Path", "Score", "Churn", "Complexity")
	local sub_separator = " " .. string.rep("─", 40) .. "┼" .. string.rep("─", 12) .. "┼" .. string.rep("─", 8) .. "┼" .. string.rep("─", 12)

	local lines = { header_title, separator, col_header, sub_separator }
	local file_mapping = {}

	for i, item in ipairs(results) do
		local name = item.file_path
		if #name > 38 then
			name = "..." .. name:sub(-35)
		end

		local line_str = string.format(" %-40s │ %10.1f │ %6d │ %10d", name, item.score, item.churn or 0, item.complexity or 0)
		table.insert(lines, line_str)
		file_mapping[#lines] = item.file_path
	end

	local footer_sep = " " .. string.rep("─", 74)
	local footer_help = " [Enter] Open  [v] VSplit  [s] Split  [t] Tab  [r] Refresh  [q] Close"
	table.insert(lines, footer_sep)
	table.insert(lines, footer_help)

	vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)

	-- Apply highlights
	vim.api.nvim_buf_clear_namespace(buf, volt_ui_ns, 0, -1)
	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Title", 0, 0, -1)
	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Comment", 1, 0, -1)
	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Bold", 2, 0, -1)
	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Comment", 3, 0, -1)

	for idx, item in ipairs(results) do
		local line_num = idx + 3
		local hl_group = get_score_hl(item.score, thresholds)
		-- Highlight file path
		vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Normal", line_num, 1, 41)
		-- Highlight score
		vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, hl_group, line_num, 43, 54)
		-- Highlight churn
		vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Number", line_num, 56, 63)
		-- Highlight complexity
		vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Special", line_num, 65, 76)
	end

	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Comment", #lines - 2, 0, -1)
	vim.api.nvim_buf_add_highlight(buf, volt_ui_ns, "Directory", #lines - 1, 0, -1)

	local width = math.min(80, (vim.o.columns or 80) - 4)
	local height = math.min(#lines + 2, (vim.o.lines or 24) - 4)
	local ui = vim.api.nvim_list_uis()[1] or { width = 80, height = 24 }

	local win_opts = {
		relative = "editor",
		width = width,
		height = height,
		col = math.floor((ui.width - width) / 2),
		row = math.floor((ui.height - height) / 2),
		style = "minimal",
		border = opts.border or "rounded",
		title = " ⚡ Volt Analysis ",
		title_pos = "center",
	}

	local win = vim.api.nvim_open_win(buf, true, win_opts)
	vim.bo[buf].modifiable = false
	vim.bo[buf].filetype = "volt_report"

	local function open_file(cmd)
		local cursor_row = vim.api.nvim_win_get_cursor(win)[1]
		local file = file_mapping[cursor_row]
		if file then
			vim.api.nvim_win_close(win, true)
			vim.cmd(cmd .. " " .. vim.fn.fnameescape(file))
		end
	end

	-- Keybindings
	vim.keymap.set("n", "<CR>", function()
		open_file("edit")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "v", function()
		open_file("vsplit")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "<C-v>", function()
		open_file("vsplit")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "s", function()
		open_file("split")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "<C-s>", function()
		open_file("split")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "t", function()
		open_file("tabnew")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "<C-t>", function()
		open_file("tabnew")
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "r", function()
		vim.api.nvim_win_close(win, true)
		if on_refresh then
			on_refresh()
		end
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "q", function()
		vim.api.nvim_win_close(win, true)
	end, { buffer = buf, silent = true, nowait = true })

	vim.keymap.set("n", "<Esc>", function()
		vim.api.nvim_win_close(win, true)
	end, { buffer = buf, silent = true, nowait = true })
end

return M
