-- Patina en Hyprland (configuración en Lua). install.sh copia este archivo a
-- ~/.local/share/patina/hyprland.lua; incluilo al final de ~/.config/hypr/hyprland.lua con:
--   dofile(os.getenv("HOME") .. "/.local/share/patina/hyprland.lua")

local patina = "@BIN@"

-- Doble Shift captura la selección. La liberación de Shift no se consume:
-- Shift sigue funcionando normal en las apps.
for _, key in ipairs({ "Shift_L", "Shift_R" }) do
    hl.bind("SHIFT + " .. key, hl.dsp.exec_cmd(patina .. " tap"), {
        release = true,
        non_consuming = true,
        description = "Patina: doble Shift captura la selección",
    })
end

hl.bind("SUPER + C", hl.dsp.exec_cmd(patina), { description = "Patina: mostrar u ocultar" })

hl.window_rule({
    name   = "patina-float",
    match  = { class = "^io\\.github\\.lautarosegura\\.Patina$" },
    float  = true,
    size   = "560 680",
    center = true,
})

-- Queda corriendo oculto para responder rápido al atajo.
hl.on("hyprland.start", function ()
    hl.exec_cmd(patina .. " --service")
end)
