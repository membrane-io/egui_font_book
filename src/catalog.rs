/// Font category: monospace or proportional.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum FontCategory {
    Monospace,
    Proportional,
}

use std::borrow::Cow;

/// A variant of a font (e.g. Regular, Italic, Light, Bold).
#[derive(Debug, Clone)]
pub struct FontVariant {
    /// Short label shown in the UI, e.g. "Italic", "Light", "Bold".
    pub label: Cow<'static, str>,
    pub url: Cow<'static, str>,
    pub cache_key: Cow<'static, str>,
}

/// A single font entry in the catalog, with one or more variants.
#[derive(Debug, Clone)]
pub struct FontEntry {
    pub name: Cow<'static, str>,
    pub category: FontCategory,
    /// First variant is the default. All entries have at least one.
    pub variants: Vec<FontVariant>,
}

impl FontEntry {
    /// The default variant's URL.
    pub fn url(&self) -> &str {
        &self.variants[0].url
    }

    /// The default variant's cache key.
    pub fn cache_key(&self) -> &str {
        &self.variants[0].cache_key
    }
}

macro_rules! v {
    ($label:literal, $slug:literal, $file:literal) => {
        FontVariant {
            label: Cow::Borrowed($label),
            url: Cow::Borrowed(concat!("https://raw.githubusercontent.com/google/fonts/main/ofl/", $slug, "/", $file)),
            cache_key: Cow::Borrowed(concat!($slug, "-", $label)),
        }
    };
}

macro_rules! mono {
    ($name:literal, $slug:literal, $file:literal $(, $($vlabel:literal => $vfile:literal),+ )? ) => {
        FontEntry {
            name: Cow::Borrowed($name),
            category: FontCategory::Monospace,
            variants: {
                #[allow(unused_mut)]
                let mut vs = vec![FontVariant {
                    label: Cow::Borrowed("Regular"),
                    url: Cow::Borrowed(concat!("https://raw.githubusercontent.com/google/fonts/main/ofl/", $slug, "/", $file)),
                    cache_key: Cow::Borrowed($name),
                }];
                $( $( vs.push(v!($vlabel, $slug, $vfile)); )+ )?
                vs
            },
        }
    };
}

macro_rules! prop {
    ($name:literal, $slug:literal, $file:literal $(, $($vlabel:literal => $vfile:literal),+ )? ) => {
        FontEntry {
            name: Cow::Borrowed($name),
            category: FontCategory::Proportional,
            variants: {
                #[allow(unused_mut)]
                let mut vs = vec![FontVariant {
                    label: Cow::Borrowed("Regular"),
                    url: Cow::Borrowed(concat!("https://raw.githubusercontent.com/google/fonts/main/ofl/", $slug, "/", $file)),
                    cache_key: Cow::Borrowed($name),
                }];
                $( $( vs.push(v!($vlabel, $slug, $vfile)); )+ )?
                vs
            },
        }
    };
}

/// Returns the curated font catalog.
///
/// Sources: Google Fonts (via their GitHub repository).
/// All fonts are SIL Open Font License or Apache 2.0.
pub fn catalog() -> Vec<FontEntry> {
    vec![
        // ── Monospace ──────────────────────────────────────────
        mono!("JetBrains Mono", "jetbrainsmono", "JetBrainsMono%5Bwght%5D.ttf",
            "Italic" => "JetBrainsMono-Italic%5Bwght%5D.ttf"
        ),
        mono!("Fira Code", "firacode", "FiraCode%5Bwght%5D.ttf"),
        mono!("Source Code Pro", "sourcecodepro", "SourceCodePro%5Bwght%5D.ttf",
            "Italic" => "SourceCodePro-Italic%5Bwght%5D.ttf"
        ),
        mono!("IBM Plex Mono", "ibmplexmono", "IBMPlexMono-Regular.ttf",
            "Thin"             => "IBMPlexMono-Thin.ttf",
            "ExtraLight"       => "IBMPlexMono-ExtraLight.ttf",
            "Light"            => "IBMPlexMono-Light.ttf",
            "Medium"           => "IBMPlexMono-Medium.ttf",
            "SemiBold"         => "IBMPlexMono-SemiBold.ttf",
            "Bold"             => "IBMPlexMono-Bold.ttf",
            "Italic"           => "IBMPlexMono-Italic.ttf",
            "ThinItalic"       => "IBMPlexMono-ThinItalic.ttf",
            "ExtraLightItalic" => "IBMPlexMono-ExtraLightItalic.ttf",
            "LightItalic"      => "IBMPlexMono-LightItalic.ttf",
            "MediumItalic"     => "IBMPlexMono-MediumItalic.ttf",
            "SemiBoldItalic"   => "IBMPlexMono-SemiBoldItalic.ttf",
            "BoldItalic"       => "IBMPlexMono-BoldItalic.ttf"
        ),
        mono!("Roboto Mono", "robotomono", "RobotoMono%5Bwght%5D.ttf",
            "Italic" => "RobotoMono-Italic%5Bwght%5D.ttf"
        ),
        mono!("Space Mono", "spacemono", "SpaceMono-Regular.ttf",
            "Bold"        => "SpaceMono-Bold.ttf",
            "Italic"      => "SpaceMono-Italic.ttf",
            "BoldItalic"  => "SpaceMono-BoldItalic.ttf"
        ),
        mono!("Inconsolata", "inconsolata", "Inconsolata%5Bwdth,wght%5D.ttf"),
        mono!("Anonymous Pro", "anonymouspro", "AnonymousPro-Regular.ttf",
            "Bold"        => "AnonymousPro-Bold.ttf",
            "Italic"      => "AnonymousPro-Italic.ttf",
            "BoldItalic"  => "AnonymousPro-BoldItalic.ttf"
        ),
        mono!("Courier Prime", "courierprime", "CourierPrime-Regular.ttf",
            "Bold"        => "CourierPrime-Bold.ttf",
            "Italic"      => "CourierPrime-Italic.ttf",
            "BoldItalic"  => "CourierPrime-BoldItalic.ttf"
        ),
        mono!("DM Mono", "dmmono", "DMMono-Regular.ttf",
            "Light"        => "DMMono-Light.ttf",
            "Medium"       => "DMMono-Medium.ttf",
            "Italic"       => "DMMono-Italic.ttf",
            "LightItalic"  => "DMMono-LightItalic.ttf",
            "MediumItalic" => "DMMono-MediumItalic.ttf"
        ),
        mono!("Red Hat Mono", "redhatmono", "RedHatMono%5Bwght%5D.ttf",
            "Italic" => "RedHatMono-Italic%5Bwght%5D.ttf"
        ),
        mono!("Overpass Mono", "overpassmono", "OverpassMono%5Bwght%5D.ttf"),
        mono!("Noto Sans Mono", "notosansmono", "NotoSansMono%5Bwdth,wght%5D.ttf"),
        mono!("Ubuntu Mono", "ubuntumono", "UbuntuMono%5Bwght%5D.ttf",
            "Italic" => "UbuntuMono-Italic%5Bwght%5D.ttf"
        ),
        mono!("Martian Mono", "martianmono", "MartianMono%5Bwdth,wght%5D.ttf"),
        mono!("Cascadia Mono", "cascadiamono", "CascadiaMono%5Bwght%5D.ttf",
            "Italic" => "CascadiaMono-Italic%5Bwght%5D.ttf"
        ),
        FontEntry {
            name: Cow::Borrowed("Commit Mono"),
            category: FontCategory::Monospace,
            variants: vec![FontVariant {
                label: Cow::Borrowed("Regular"),
                url: Cow::Borrowed("https://raw.githubusercontent.com/eigilnikolajsen/commit-mono/main/src/fonts/fontlab/CommitMonoV143-VF.ttf"),
                cache_key: Cow::Borrowed("Commit Mono"),
            }],
        },

        // ── Proportional ───────────────────────────────────────
        prop!("Roboto", "roboto", "Roboto%5Bwdth,wght%5D.ttf",
            "Italic" => "Roboto-Italic%5Bwdth,wght%5D.ttf"
        ),
        prop!("Cascadia Code", "cascadiacode", "CascadiaCode%5Bwght%5D.ttf",
            "Italic" => "CascadiaCode-Italic%5Bwght%5D.ttf"
        ),
        prop!("Inter", "inter", "Inter%5Bopsz,wght%5D.ttf",
            "Italic" => "Inter-Italic%5Bopsz,wght%5D.ttf"
        ),
        prop!("Open Sans", "opensans", "OpenSans%5Bwdth,wght%5D.ttf",
            "Italic" => "OpenSans-Italic%5Bwdth,wght%5D.ttf"
        ),
        prop!("Lato", "lato", "Lato-Regular.ttf",
            "Thin"              => "Lato-Thin.ttf",
            "ExtraLight"        => "Lato-ExtraLight.ttf",
            "Light"             => "Lato-Light.ttf",
            "Medium"            => "Lato-Medium.ttf",
            "SemiBold"          => "Lato-SemiBold.ttf",
            "Bold"              => "Lato-Bold.ttf",
            "ExtraBold"         => "Lato-ExtraBold.ttf",
            "Black"             => "Lato-Black.ttf",
            "Italic"            => "Lato-Italic.ttf",
            "ThinItalic"        => "Lato-ThinItalic.ttf",
            "ExtraLightItalic"  => "Lato-ExtraLightItalic.ttf",
            "LightItalic"       => "Lato-LightItalic.ttf",
            "MediumItalic"      => "Lato-MediumItalic.ttf",
            "SemiBoldItalic"    => "Lato-SemiBoldItalic.ttf",
            "BoldItalic"        => "Lato-BoldItalic.ttf",
            "ExtraBoldItalic"   => "Lato-ExtraBoldItalic.ttf",
            "BlackItalic"       => "Lato-BlackItalic.ttf"
        ),
        prop!("Montserrat", "montserrat", "Montserrat%5Bwght%5D.ttf",
            "Italic" => "Montserrat-Italic%5Bwght%5D.ttf"
        ),
        prop!("Nunito", "nunito", "Nunito%5Bwght%5D.ttf",
            "Italic" => "Nunito-Italic%5Bwght%5D.ttf"
        ),
        prop!("Raleway", "raleway", "Raleway%5Bwght%5D.ttf",
            "Italic" => "Raleway-Italic%5Bwght%5D.ttf"
        ),
        prop!("Work Sans", "worksans", "WorkSans%5Bwght%5D.ttf",
            "Italic" => "WorkSans-Italic%5Bwght%5D.ttf"
        ),
        prop!("IBM Plex Sans", "ibmplexsans", "IBMPlexSans%5Bwdth,wght%5D.ttf",
            "Italic" => "IBMPlexSans-Italic%5Bwdth,wght%5D.ttf"
        ),
        prop!("Outfit", "outfit", "Outfit%5Bwght%5D.ttf"),
        prop!("Space Grotesk", "spacegrotesk", "SpaceGrotesk%5Bwght%5D.ttf"),
        prop!("Plus Jakarta Sans", "plusjakartasans", "PlusJakartaSans%5Bwght%5D.ttf",
            "Italic" => "PlusJakartaSans-Italic%5Bwght%5D.ttf"
        ),
        prop!("DM Sans", "dmsans", "DMSans%5Bopsz,wght%5D.ttf",
            "Italic" => "DMSans-Italic%5Bopsz,wght%5D.ttf"
        ),
        prop!("Rubik", "rubik", "Rubik%5Bwght%5D.ttf",
            "Italic" => "Rubik-Italic%5Bwght%5D.ttf"
        ),
        prop!("Karla", "karla", "Karla%5Bwght%5D.ttf",
            "Italic" => "Karla-Italic%5Bwght%5D.ttf"
        ),
        prop!("Manrope", "manrope", "Manrope%5Bwght%5D.ttf"),
        prop!("Sora", "sora", "Sora%5Bwght%5D.ttf"),
        prop!("Lexend", "lexend", "Lexend%5Bwght%5D.ttf"),
        prop!("Figtree", "figtree", "Figtree%5Bwght%5D.ttf",
            "Italic" => "Figtree-Italic%5Bwght%5D.ttf"
        ),
        prop!("Geist", "geist", "Geist%5Bwght%5D.ttf"),
        prop!("Atkinson Hyperlegible Next", "atkinsonhyperlegiblenext", "AtkinsonHyperlegibleNext%5Bwght%5D.ttf",
            "Italic" => "AtkinsonHyperlegibleNext-Italic%5Bwght%5D.ttf"
        ),
    ]
}
