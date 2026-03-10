use crate::MyApp;
use eframe::epaint::text::FontData;
use std::collections::HashMap;
use std::sync::Arc;

impl MyApp {
    // Initialize egui fonts. Note that this function globally modifies egui settings.
    pub(crate) fn setup_fonts(cc: &eframe::CreationContext<'_>) {
        // egui comes with some default fonts, but they don't cover all characters so we
        // need to add fallbacks fonts.
        // See https://docs.rs/egui/latest/egui/struct.FontDefinitions.html

        // Fonts are stored within the binary.
        // The current font data is Jetbrains Mono:
        //   curl -L -o assets/fonts/JetBrainsMono-Regular.ttf https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/ttf/JetBrainsMono-Regular.ttf
        //   curl -L -o assets/fonts/JetBrainsMono-Bold.ttf https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/ttf/JetBrainsMono-Bold.ttf
        //   curl -L -o assets/fonts/JetBrainsMono-Italic.ttf https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/ttf/JetBrainsMono-Italic.ttf
        //   curl -L -o assets/fonts/JetBrainsMono-BoldItalic.ttf https://github.com/JetBrains/JetBrainsMono/raw/master/fonts/ttf/JetBrainsMono-BoldItalic.ttf
        // Note that JetBrainsMono is licenced: https://www.jetbrains.com/lp/mono/ https://github.com/JetBrains/JetBrainsMono/blob/master/OFL.txt
        //   but it's it's free to use as long as I don't sell or distribute the font itself directly. It doesn't need to be credited.
        //
        // Another option would be to use the default system font: https://github.com/emilk/egui/discussions/1344
        // but I don't like that approach since it depends on something outside of this binary.
        const MY_FONTS_DATA: [(&str, &[u8]); 4] = [
            (
                "JetBrainsMono-Regular",
                include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
            ),
            (
                "JetBrainsMono-Bold",
                include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf"),
            ),
            (
                "JetBrainsMono-Italic",
                include_bytes!("../../assets/fonts/JetBrainsMono-Italic.ttf"),
            ),
            (
                "JetBrainsMono-BoldItalic",
                include_bytes!("../../assets/fonts/JetBrainsMono-BoldItalic.ttf"),
            ),
        ];

        // Load all font variants. Note that this loads an entirely separate font for each bold/italic/etc
        // but then uses them in the same way. This is unintuitive but intentional. egui chooses
        // the first font to match a request, so if you try to print italicized text it'll iterate
        // through loaded fonts until it finds the `italic` ttf. So as long as we push these
        // into egui's font registry, they will be used when appropriate.
        let mut my_fonts: HashMap<&str, Arc<FontData>> = HashMap::new();
        for (name, data) in &MY_FONTS_DATA {
            my_fonts.insert(name, Arc::from(FontData::from_static(data)));
        }

        // Get the built-in defaults, and add my fonts to it. This maps the name to the data.
        let mut fonts = egui::FontDefinitions::default();
        for (name, data) in my_fonts {
            fonts.font_data.insert(name.to_string(), data);
        }

        // To use my fonts as fallback to existing fonts, the names need to be added to a PATH-like list.
        // egui will iterate over the list until it finds a font that is valid.
        for (name, _) in &MY_FONTS_DATA {
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .push(name.to_string());

            // WARNING: JetBrainsMono is a monospace font so it shouldn't really be used here, but
            //   it's safe enough since the default fonts should handle almost everything.
            //   This fallback should only be used for things like emojis.
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .push(name.to_string());
        }

        // Apply the font configuration
        cc.egui_ctx.set_fonts(fonts);
    }
}
