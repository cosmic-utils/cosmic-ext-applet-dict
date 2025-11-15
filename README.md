# Dictionary Applet (for the COSMIC™ Desktop)

An English language dictionary applet, perfect for quick access searching for words and definitions, or cycling through some random words to enrich your knowledge of the English language!

## Manual installation

You're going to need to make sure you have the ability to compile Rust binaries, along with `git` and `just`

```sh
git clone https://github.com/cappsyco/cosmic-ext-applet-dict && cd cosmic-ext-applet-dict
just build-release
sudo just install
```

## Flatpak installation (no yet active)

By far the best way to install the Logo Menu is through the official COSMIC™ Flatpak repository. Firstly, ensure you have Flatpak itself installed. You then should be able to search for and install Logo Menu from the COSMIC™ Store, under the Applets category. Alternatively, you can ensure you have the correct repo enabled and install through the command line.

```sh
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install dev.cappsy.CosmicExtAppletDict
```

## With thanks & Credit
* [Open-Source English Dictionary](https://github.com/CloudBytes-Academy/English-Dictionary-Open-Source) - Public Domain based dictionary that the applet is currently based on
* [Book icon used for the logo and applet icon](https://www.svgrepo.com/svg/533406/book) by [Dazzle Ui](https://www.svgrepo.com/author/Dazzle%20UI/)
* [System76 and their COSMIC desktop environment](https://system76.com/cosmic/)
* [COSMIC Utilities](https://github.com/cosmic-utils/) - Organization containing third party utilities for COSMIC™
