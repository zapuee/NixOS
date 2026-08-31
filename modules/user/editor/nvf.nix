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

        # Nix language 
        languages.nix = {
          enable = true;
          treesitter.enable = true;
          format.enable = false;
          lsp = {
            enable = true;
            servers = [ "nixd" ];
          };
        };

        # Lua language
        languages.lua = {
          enable = true;
          treesitter.enable = true;
          format.enable = false;
          lsp = {
            enable = true;
            servers = [ "lua-language-server" ];
          };
        };

        # Rust language
        languages.rust = {
          enable = true;
          treesitter.enable = true;
          format.enable = false;
          lsp = {
            enable = true;
            servers = [ "rust-analyzer" ];
          };
        };

        # Neo-tree
        filetree.neo-tree = {
          enable = true;
        };

        # Fuzzy Find 
        fzf-lua.enable = true;

        utility.surround.enable = true;

        # Show indent
        visuals.indent-blankline.enable = true;

        # Colors!
        ui.nvim-highlight-colors.enable = true;

        # Automatic Tab Size
        utility.sleuth.enable = true;

        # Harpoon
        navigation.harpoon.enable = true;

        # Lualine
        statusline.lualine.enable = true;

        # Autoclose
        autopairs.nvim-autopairs.enable = true;

        # Blink completion
        autocomplete.blink-cmp = {
          enable = true;
        
          setupOpts = {
            keymap = {
              preset = "default";
        
              "<C-n>" = [ "select_next" ];
              "<C-p>" = [ "select_prev" ];
            };
          };
        };

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

        globals = {
          mapleader = "\\";
          maplocalleader = "\\";
        };

        keymaps = [
          {
            mode = "n";
            key = "<CR>";
            action = "o<Esc>";
          }

          {
            mode = "n";
            key = "<leader>d";
            action = "<cmd>lua vim.diagnostic.open_float()<CR>";
            desc = "Show current diagnostic";
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
        
          -- fzf-lua
          local fzf = require("fzf-lua")
        
          vim.keymap.set("n", "<leader>ff", fzf.files, { desc = "Find Files" })
          vim.keymap.set("n", "<leader>fg", fzf.live_grep, { desc = "Live Grep" })
          vim.keymap.set("n", "<leader>fb", fzf.buffers, { desc = "Find Buffers" })
          vim.keymap.set("n", "<leader>fh", fzf.help_tags, { desc = "Help Tags" })
        
          -- Harpoon
          local harpoon = require("harpoon")
        
          vim.keymap.set("n", "<leader>ha", function()
            harpoon:list():add()
          end, { desc = "Harpoon Add" })
        
          vim.keymap.set("n", "<leader>hh", function()
            harpoon.ui:toggle_quick_menu(harpoon:list())
          end, { desc = "Harpoon Menu" })
        
          vim.keymap.set("n", "<leader>h1", function()
            harpoon:list():select(1)
          end, { desc = "Harpoon File 1" })
        
          vim.keymap.set("n", "<leader>h2", function()
            harpoon:list():select(2)
          end, { desc = "Harpoon File 2" })
        
          vim.keymap.set("n", "<leader>h3", function()
            harpoon:list():select(3)
          end, { desc = "Harpoon File 3" })
        
          vim.keymap.set("n", "<leader>h4", function()
            harpoon:list():select(4)
          end, { desc = "Harpoon File 4" })

          vim.keymap.set("n", "<leader>t", "<cmd>Neotree toggle<CR>", {
            desc = "Neo-tree",
          })
        '';
      };
    };
  };
}
