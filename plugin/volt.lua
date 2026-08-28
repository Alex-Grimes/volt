vim.api.nvim_create_user_command("VoltScan", function()
	require("volt").scan_project()
end, { desc = "Scan project for voltage hotspots" })

vim.api.nvim_create_user_command("VoltSummary", function()
	require("volt").show_summary()
end, { desc = "Display floating voltage report" })

vim.api.nvim_create_user_command("VoltRefresh", function()
	require("volt").scan_project()
end, { desc = "Refresh voltage hotspot analysis" })

vim.api.nvim_create_user_command("VoltClear", function()
	require("volt").clear()
end, { desc = "Clear all Volt signs and virtual text annotations" })

vim.api.nvim_create_user_command("VoltPicker", function()
	require("volt").picker()
end, { desc = "Open configured or detected picker (Snacks or Telescope)" })

vim.api.nvim_create_user_command("VoltSnacks", function()
	require("volt").snacks()
end, { desc = "Open Snacks picker with Volt hotspots" })

vim.api.nvim_create_user_command("VoltTelescope", function()
	require("volt").telescope()
end, { desc = "Open Telescope picker with Volt hotspots" })
