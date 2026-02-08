# Dictionary Applet (for the COSMIC™ Desktop)

An English language dictionary applet, perfect for quick access searching for words and definitions, or cycling through some random words to enrich your knowledge of the English language!

We use [our own fork of the Wordset Dictionary](https://github.com/cappsyco/wordset-dictionary), so please submit any additions or corrections there so we can guarantee these changes make it into the applet.

![A view of the open applet showing the search box, some results and the random link at the bottom.](https://raw.githubusercontent.com/cappsyco/cosmic-ext-applet-dict/main/resources/screenshots/screen1.jpg)

## Flatpak installation

By far the best way to install the Logo Menu is through the official COSMIC™ Flatpak repository. Firstly, ensure you have Flatpak itself installed. You then should be able to search for and install Logo Menu from the COSMIC™ Store, under the Applets category. Alternatively, you can ensure you have the correct repo enabled and install through the command line.

```sh
flatpak remote-add --if-not-exists --user cosmic https://apt.pop-os.org/cosmic/cosmic.flatpakrepo
flatpak install dev.cappsy.CosmicExtAppletDict
```

## Arch User Repository installation

The applet can be installed directly from [the AUR](https://aur.archlinux.org/packages/cosmic-ext-applet-dict-git), and this will get you very latest code and not be tied to tagged releases. You will need `base-devel` and `git` if you don't have them already.

```sh
sudo pacman -S base-devel git
git clone https://aur.archlinux.org/cosmic-ext-applet-logomenu-git.git
cd cosmic-ext-applet-logomenu-git && makepkg -si
```

## Manual installation

You're going to need to make sure you have the ability to compile Rust binaries, along with `git` and `just`

```sh
git clone https://github.com/cappsyco/cosmic-ext-applet-dict && cd cosmic-ext-applet-dict
just build-release
sudo just install
```

## Credit & thanks
* [Wordset Dictionary](https://github.com/wordset/wordset-dictionary) - Open source, collaborative dictionary on which this applet is currently based
* [Book icon used for the logo and applet icon](https://www.svgrepo.com/svg/533406/book) by [Dazzle Ui](https://www.svgrepo.com/author/Dazzle%20UI/)
* [System76 and their COSMIC desktop environment](https://system76.com/cosmic/)
* [COSMIC Utilities](https://github.com/cosmic-utils/) - Organization containing third party utilities for COSMIC™
