{ lib, config, inputs, ... }:

{
  imports = [
    inputs.nvf.homeManagerModules.default
  ];

  config = lib.mkIf (config.userSettings.editor == "nvf") {
    programs.nvf = {
      enable = true;

      settings.vim = {
        clipboard = {
          enable = true;
          registers = "unnamedplus";
          providers = {
            wl-copy.enable = true;
          };
        };

        options = {
          number = true;
          relativenumber = true;
          shiftwidth = 4;
          tabstop = 4;
          softtabstop = 4;
          expandtab = true;
          completeopt = "menuone,noselect";
        };

        theme = {
          enable = true;
          name = "tokyonight";
          style = "moon";
          transparent = true;
        };

        # Nix language + LSP + Treesitter
        languages.nix = {
          enable = true;
          lsp.enable = true;
          treesitter.enable = true;
          format.enable = false;
        };

        # Lua language + LSP + Treesitter
        languages.lua = {
          enable = true;
          lsp.enable = true;
          treesitter.enable = true;
        };

        # Explicit language servers
        languages.nix.lsp.servers = [ "nixd" ];
        languages.lua.lsp.servers = [ "lua-language-server" ];

        # Neo-tree
        filetree.neo-tree = {
          enable = true;
        };

        # Automatic Tab Size
        utility.sleuth.enable = true;

        # Telescope
        telescope.enable = true;

        # Harpoon
        navigation.harpoon.enable = true;

        # Lualine
        statusline.lualine.enable = true;

        # Autoclose
        autopairs.nvim-autopairs.enable = true;

        # Blink completion
        autocomplete.blink-cmp.enable = true;

        # Highlight
        highlight = {
          Normal = {
            fg = "#d0d0d0";
            bg = "#1e1e1e";
          };

          NormalNC = {
            fg = "#d0d0d0";
            bg = "#1e1e1e";
          };

          SignColumn = {
            bg = "#1e1e1e";
          };

          LineNr = {
            fg = "#777777";
            bg = "#1e1e1e";
          };

          CursorLineNr = {
            fg = "#b8b8b8";
            bg = "#1e1e1e";
          };

          Comment = {
            fg = "#686868";
            bg = "#1e1e1e";
          };
        };

        # UI helpers
        ui = {
          borders.enable = true;
          illuminate.enable = true;
        };

        keymaps = [
          {
            mode = "n";
            key = "<CR>";
            action = "o<Esc>";
          }
        ];

        luaConfigPost = ''
          vim.g.loaded_node_provider = 0
          vim.g.loaded_perl_provider = 0
          vim.g.loaded_ruby_provider = 0
          vim.g.loaded_python3_provider = 0

          vim.keymap.set("n", "<Tab>", function()
            local row, col = unpack(vim.api.nvim_win_get_cursor(0))
            local indent = string.rep(" ", vim.bo.shiftwidth)

            vim.api.nvim_buf_set_text(
              0,
              row - 1,
              col,
              row - 1,
              col,
              { indent }
            )

            vim.api.nvim_win_set_cursor(0, { row, col + #indent })
          end)
        '';
      };
    };
  };
}
