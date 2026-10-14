-- Patina for Hyprland 0.55+ (hyprland.lua). Include it at the end of ~/.config/hypr/hyprland.lua
-- with the path to your clone, for example:
--   dofile(os.getenv("HOME") .. "/code/patina/hypr/patina.lua")
-- If the command isn't in ~/.local/bin, set PATINA_BIN = "/path/to/patina" before the dofile.

local patina = PATINA_BIN or (os.getenv("HOME") .. "/.local/bin/patina")

-- Double Shift captures the selection. The Shift release isn't consumed, so Shift keeps
-- working normally in apps.
for _, key in ipairs({ "Shift_L", "Shift_R" }) do
    hl.bind("SHIFT + " .. key, hl.dsp.exec_cmd(patina .. " tap"), {
        release = true,
        non_consuming = true,
        description = "Patina: double Shift captures the selection",
    })
end

hl.bind("SUPER + C", hl.dsp.exec_cmd(patina), { description = "Patina: show or hide" })

hl.window_rule({
    name   = "patina-float",
    match  = { class = "^io\\.github\\.lautarosegura\\.Patina$" },
    float  = true,
    size   = "560 680",
    center = true,
})

-- Keeps running hidden so the shortcut responds instantly.
hl.on("hyprland.start", function ()
    hl.exec_cmd(patina .. " --service")
end)
