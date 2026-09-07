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
          showtabline = 1;
        };

        theme = {
          enable = true;
          name = "tokyonight";
          style = "storm";
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

        # tmux navigation
        utility.smart-splits.enable = true;

        # Because im a noobini pizzanini
        binds.whichKey.enable = true;
        ui.borders.plugins.which-key.enable = true;

        # Navigation
        utility.motion.flash-nvim.enable = true;

        # Fuzzy Find
        fzf-lua.enable = true;

        utility.surround.enable = true;

        # lsp info
        visuals.fidget-nvim.enable = true;

        # Show indent
        visuals.indent-blankline.enable = true;

        # Splitjoin
        mini.splitjoin.enable = true;

        # Nice scrollbar
        visuals.satellite-nvim.enable = true;

        # Show CursorLine
        visuals.nvim-cursorline.enable = true;
        visuals.nvim-cursorline.setupOpts.cursorline.enable = true;

        # Colors!
        ui.nvim-highlight-colors.enable = true;

        # Automatic Tab Size
        utility.sleuth.enable = true;

        # Harpoon
        navigation.harpoon = {
          enable = true;

          mappings = {
            markFile = "<leader>ea";
            listMarks = "<leader>ee";
            file1 = "<leader>e1";
            file2 = "<leader>e2";
            file3 = "<leader>e3";
            file4 = "<leader>e4";
          };
        };

        # Lualine
        statusline.lualine.enable = true;

        # Autoclose
        autopairs.nvim-autopairs.enable = true;

        # Blink completion
        autocomplete.blink-cmp = {
          enable = true;

          setupOpts = {
            completion.menu.border = "rounded";

            keymap = {
              preset = "default";

              "<C-n>" = [ "select_next" ];
              "<C-p>" = [ "select_prev" ];
            };
          };
        };

        # Highlight
        highlight = {
          IblIndent = {
            fg = "#30353f";
          };

          IblScope = {
            fg = "#3a414d";
          };

          BlinkCmpMenu = {
            fg = "#c7ccd6";
            bg = "#292e38";
          };

          BlinkCmpMenuBorder = {
            fg = "#343a46";
            bg = "#292e38";
          };

          BlinkCmpMenuSelection = {
            fg = "#d6d9e0";
            bg = "#353b47";
          };

          # Optional: make the secondary text more subdued
          BlinkCmpLabelDetail = {
            fg = "#737b89";
          };

          BlinkCmpLabelDescription = {
            fg = "#7d8594";
          };

          NormalFloat = {
            fg = "#c7ccd6";
            bg = "#292e38";
          };

          FloatBorder = {
            fg = "#343a46";
            bg = "#292e38";
          };

          Normal = {
            fg = "#d4d7de";
            bg = "#202329";
          };

          NormalNC = {
            fg = "#d4d7de";
            bg = "#202329";
          };

          SignColumn = {
            bg = "#202329";
          };

          LineNr = {
            fg = "#59616e";
            bg = "#202329";
          };

          CursorLine = {
            bg = "#292e38";
          };

          CursorLineNr = {
            fg = "#aeb7c5";
            bg = "#292e38";
          };

          Comment = {
            fg = "#626b78";
            bg = "#202329";
          };

          StatusLine = {
            fg = "#aeb7c5";
            bg = "#292e38";
          };

          VertSplit = {
            fg = "#343a46";
            bg = "#202329";
          };

          WinSeparator = {
            fg = "#343a46";
            bg = "#202329";
          };

          Pmenu = {
            fg = "#d4d7de";
            bg = "#292e38";
          };

          PmenuSel = {
            fg = "#ffffff";
            bg = "#3a4352";
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
            key = "t";
            mode = "n";
            action = ":tabnew<CR>";
            silent = true;
          }

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

          vim.keymap.set("n", "<leader>t", "<cmd>Neotree toggle<CR>", {
            desc = "Neo-tree",
          })
        '';
      };
    };
  };
}
