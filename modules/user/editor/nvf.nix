{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:

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

        languages = {
          nix = {
            enable = true;
            treesitter.enable = true;
            format.enable = false;
            lsp = {
              enable = true;
              servers = [ "nixd" ];
            };
          };

          lua = {
            enable = true;
            treesitter.enable = true;
            format.enable = false;
            lsp = {
              enable = true;
              servers = [ "lua-language-server" ];
            };
          };

          rust = {
            enable = true;
            treesitter.enable = true;
            format.enable = false;
            lsp = {
              enable = true;
              servers = [ "rust-analyzer" ];
            };
          };
        };

        lsp = {
          enable = true;
          servers.luau-lsp = {
            cmd = [
              (lib.getExe pkgs.luau-lsp)
              "lsp"
            ];

            filetypes = [
              "luau"
            ];

            root_markers = [
              ".git"
              ".luaurc"
            ];
          };
        };

        lsp.servers."rust-analyzer".settings."rust-analyzer" = {
          cargo = {
            allTargets = false;
          };
          check = {
            allTargets = false;
          };
          cachePriming = {
            enable = true;
            numThreads = 4;
          };
        };

        # Neo-tree
        filetree.neo-tree = {
          enable = true;
        };

        utility = {
          smart-splits.enable = true;
          motion.flash-nvim.enable = true;
          surround.enable = true;
          sleuth.enable = true;
        };

        # Because im a noobini pizzanini
        binds.whichKey.enable = true;
        ui = {
          borders = {
            enable = true;
            plugins.which-key.enable = true;
          };
          nvim-highlight-colors.enable = true;
          illuminate.enable = true;
        };

        # Fuzzy Find
        fzf-lua.enable = true;

        terminal.toggleterm = {
          enable = true;

          setupOpts = {
            direction = "horizontal";
            size = 12;
          };
        };

        visuals = {
          fidget-nvim.enable = true;
          indent-blankline.enable = true;
          satellite-nvim.enable = true;
          nvim-cursorline = {
            enable = true;
            setupOpts.cursorline.enable = true;
          };
        };

        # Splitjoin
        mini.splitjoin.enable = true;

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
            fuzzy = {
              implementation = "rust";
              prebuilt_binaries = {
                download = false;
              };
            };
            completion.menu.border = "rounded";

            signature = {
              enabled = true;
              trigger = {
                enabled = false;
              };
              window = {
                show_documentation = false;
              };
            };

            keymap = {
              preset = "default";
              "<C-n>" = [ "select_next" ];
              "<C-p>" = [ "select_prev" ];
              "<S-CR>" = [
                (lib.generators.mkLuaInline ''
                  function(cmp)
                    local item = cmp.get_selected_item()
                    if not item then
                      return false
                    end

                    local text_edits = require("blink.cmp.lib.text_edits")
                    local edit = text_edits.get_from_item(item)

                    local kinds = vim.lsp.protocol.CompletionItemKind

                    local callable =
                      item.kind == kinds.Function
                      or item.kind == kinds.Method
                      or item.kind == kinds.Constructor

                    local label = item.label or ""
                    local filter = item.filterText

                    local text = label

                    if filter
                      and filter ~= ""
                      and label:find(filter, 1, true)
                    then
                      text = filter
                    end

                    if callable then
                      text = text:gsub("%s*%b()%s*$", "")
                    end

                    edit.newText = text

                    cmp.cancel({
                      callback = function()
                        text_edits.apply(edit)

                        vim.api.nvim_win_set_cursor(0, {
                          edit.range.start.line + 1,
                          edit.range.start.character + #text,
                        })
                      end,
                    })

                    return true
                  end
                '')
                "fallback"
              ];
            };
          };
        };

        lazy.plugins."${pkgs.vimPlugins.tiny-inline-diagnostic-nvim.pname}" = {
          package = pkgs.vimPlugins.tiny-inline-diagnostic-nvim;
          event = [ "LspAttach" ];
          setupModule = "tiny-inline-diagnostic";
          setupOpts = {
            preset = "modern";
            options = {
              show_source.enabled = true;
              multilines = {
                enabled = true;
                always_show = false;
              };
              overflow.mode = "wrap";
              add_messages.display_count = true;
              throttle = 50;
              virt_texts.priority = 2048;
            };
          };
          after = ''
            vim.diagnostic.config({
              virtual_text = false,
              underline = true,
              update_in_insert = false,
              severity_sort = true,
            })
          '';
        };

        # Highlight
        highlight = {
          FlashLabel = {
            fg = "#1e1e2e";
            bg = "#89b4fa";
            bold = true;
          };

          FlashMatch = {
            fg = "#cdd6f4";
            bg = "#45475a";
          };

          FlashCurrent = {
            fg = "#1e1e2e";
            bg = "#a6e3a1";
            bold = true;
          };

          FlashBackdrop = {
            fg = "#585b70";
          };

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

        globals = {
          mapleader = "\\";
          maplocalleader = "\\";

          loaded_node_provider = 0;
          loaded_perl_provider = 0;
          loaded_ruby_provider = 0;
          loaded_python3_provider = 0;
        };

        keymaps = [
          {
            mode = [
              "n"
              "t"
            ];
            key = "<leader>t";
            action = "<cmd>ToggleTerm<CR>";
            desc = "Toggle Terminal";
          }

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

          {
            mode = "n";
            key = "<leader>ff";
            action = "<cmd>FzfLua files<CR>";
            desc = "Find Files";
          }

          {
            mode = "n";
            key = "<leader>fg";
            action = "<cmd>FzfLua live_grep<CR>";
            desc = "Live Grep";
          }

          {
            mode = "n";
            key = "<leader>fb";
            action = "<cmd>FzfLua buffers<CR>";
            desc = "Find Buffers";
          }

          {
            mode = "n";
            key = "<leader>fh";
            action = "<cmd>FzfLua help_tags<CR>";
            desc = "Help Tags";
          }

          {
            key = "\\h";
            mode = "n";
            action = ''
              function()
                local enabled = vim.lsp.inlay_hint.is_enabled({ bufnr = 0 })
                vim.lsp.inlay_hint.enable(not enabled, { bufnr = 0 })
              end
            '';
            lua = true;
            desc = "Toggle LSP hints";
          }
        ];

        luaConfigPost = ''
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

          vim.keymap.set("n", "<leader>n", "<cmd>Neotree toggle<CR>", {
            desc = "Neo-tree",
          })
        '';
      };
    };
  };
}
