local M = {}

function M.picker(results, opts)
	opts = opts or {}
	local ok_telescope, telescope = pcall(require, "telescope")
	if not ok_telescope then
		vim.notify("Volt: 'telescope.nvim' is required for :VoltPicker.", vim.log.levels.WARN)
		return
	end

	local pickers = require("telescope.pickers")
	local finders = require("telescope.finders")
	local conf = require("telescope.config").values
	local entry_display = require("telescope.pickers.entry_display")

	local displayer = entry_display.create({
		separator = " ",
		items = {
			{ width = 2 },
			{ width = 10 },
			{ remaining = true },
		},
	})

	local make_display = function(entry)
		local score_str = string.format("%6.1f", entry.value.score)
		local details = string.format("%s (churn: %d, complexity: %d)", entry.value.file_path, entry.value.churn or 0, entry.value.complexity or 0)
		return displayer({
			{ "⚡", "DiagnosticWarn" },
			{ score_str, "Title" },
			{ details, "Comment" },
		})
	end

	pickers.new(opts, {
		prompt_title = "⚡ Volt Hotspots",
		finder = finders.new_table({
			results = results,
			entry_maker = function(item)
				return {
					value = item,
					display = make_display,
					ordinal = string.format("%f %s", item.score, item.file_path),
					path = item.file_path,
				}
			end,
		}),
		sorter = conf.generic_sorter(opts),
		previewer = conf.file_previewer(opts),
	}):find()
end

return M
