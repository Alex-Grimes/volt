vim.api.nvim_create_user_command("VoltScan", function()
	require("volt").scan_project()
end, {})
