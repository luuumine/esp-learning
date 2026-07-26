local rust_analyzer = {
	cargo = {
		target = "xtensa-esp32-none-elf",
		allTargets = false,
	},
}

vim.lsp.config("rust_analyzer", {
	settings = {
		["rust-analyzer"] = rust_analyzer,
	},
})
