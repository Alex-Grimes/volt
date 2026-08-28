local M = {}

function M.picker(results, opts)
	opts = opts or {}
	local ok, snacks = pcall(require, "snacks")
	local picker_api = (ok and snacks.picker) or (_G.Snacks and _G.Snacks.picker)

	if not picker_api then
		vim.notify("Volt: 'snacks.nvim' (with picker enabled) is required for snacks picker.", vim.log.levels.WARN)
		return
	end

	local items = {}
	for _, item in ipairs(results or {}) do
		table.insert(items, {
			file = item.file_path,
			text = string.format("%f %s", item.score, item.file_path),
			score = item.score,
			churn = item.churn or 0,
			complexity = item.complexity or 0,
		})
	end

	picker_api({
		title = "⚡ Volt Hotspots",
		items = items,
		format = function(item, _picker)
			local score_str = string.format("%6.1f", item.score)
			local details = string.format(" (churn: %d, complexity: %d)", item.churn, item.complexity)
			return {
				{ "⚡ ", "DiagnosticWarn" },
				{ score_str .. " ", "Title" },
				{ item.file, "Normal" },
				{ details, "Comment" },
			}
		end,
		confirm = function(picker, item)
			picker:close()
			if item and item.file then
				vim.cmd("edit " .. vim.fn.fnameescape(item.file))
			end
		end,
	})
end

return M
