include /usr/share/dpkg/default.mk

PACKAGE=pve-yew-mobile-gui

BUILDDIR ?= $(PACKAGE)-$(DEB_VERSION_UPSTREAM)
ORIG_SRC_TAR=$(PACKAGE)_$(DEB_VERSION_UPSTREAM).orig.tar.gz

DEB=$(PACKAGE)_$(DEB_VERSION)_$(DEB_HOST_ARCH).deb
DSC=$(PACKAGE)_$(DEB_VERSION).dsc

# TODO: adapt for yew ui
CARGO ?= cargo
ifeq ($(BUILD_MODE), release)
CARGO_BUILD_ARGS += --release
COMPILEDIR := target/release
else
COMPILEDIR := target/debug
endif

DESTDIR =
PREFIX = /usr
UIDIR = $(PREFIX)/share/pve-yew-mobile-gui

COMPILED_OUTPUT := \
	dist/pve-yew-mobile-gui_bundle.js \
	dist/pve-yew-mobile-gui_bg.wasm.gz

all: $(COMPILED_OUTPUT)

dist:
	mkdir dist

dist/pve-yew-mobile-gui.js dist/pve-yew-mobile-gui_bg.wasm &: $(shell find src -name '*.rs')
	proxmox-wasm-builder build -n pve-yew-mobile-gui --release --optimize

.PHONY: rebuild
rebuild:
	proxmox-wasm-builder build -n pve-yew-mobile-gui --release

dist/pve-yew-mobile-gui_bundle.js: dist/pve-yew-mobile-gui.js dist/pve-yew-mobile-gui_bg.wasm
	esbuild --bundle dist/pve-yew-mobile-gui.js --format=esm >dist/pve-yew-mobile-gui_bundle.js.tmp
	mv dist/pve-yew-mobile-gui_bundle.js.tmp dist/pve-yew-mobile-gui_bundle.js

dist/pve-yew-mobile-gui_bg.wasm.gz: dist/pve-yew-mobile-gui_bg.wasm
	gzip -c9 $^ > $@

dist/mobile-yew-style.css: pwt-assets/scss/mobile-yew-style.scss
	rust-grass $< $@

dist/crisp-yew-style.css: pwt-assets/scss/crisp-yew-style.scss
	rust-grass $< $@

install-assets: index.html.tpl manifest.json dist/mobile-yew-style.css dist/crisp-yew-style.css
	install -dm0755 $(DESTDIR)$(UIDIR)
	install -dm0755 $(DESTDIR)$(UIDIR)/js

	install -dm0755 $(DESTDIR)$(UIDIR)/images
	#install -m0644 images/favicon.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/icon-cd-drive.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/icon-cpu.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/icon-memory.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo_icon_black.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo_icon_white.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox-icon-512.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox-icon-192.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/screenshot-dashboard.png $(DESTDIR)$(UIDIR)/images

	install -dm0755 $(DESTDIR)$(UIDIR)/fonts
	install -dm0755 $(DESTDIR)$(UIDIR)/css

	install -m0644 pwt-assets/assets/font-awesome.css $(DESTDIR)$(UIDIR)/css
	install -m0644 pwt-assets/assets/fonts/RobotoFlexVariableFont.woff2 $(DESTDIR)$(UIDIR)/fonts
	install -m0644 pwt-assets/assets/fonts/fontawesome-webfont.woff2 $(DESTDIR)$(UIDIR)/fonts

	install -m0644 index.html.tpl $(DESTDIR)$(UIDIR)
	install -m0644 manifest.json $(DESTDIR)$(UIDIR)
	install -m0644 pve.css $(DESTDIR)$(UIDIR)/css
	install -m0644 dist/mobile-yew-style.css $(DESTDIR)$(UIDIR)/css
	install -m0644 dist/crisp-yew-style.css $(DESTDIR)$(UIDIR)/css

install: $(COMPILED_OUTPUT) install-assets
	install -m0644 dist/pve-yew-mobile-gui_bundle.js $(DESTDIR)$(UIDIR)/js
	#fixme: install/use .gziped .wasm file
	install -m0644 dist/pve-yew-mobile-gui_bg.wasm $(DESTDIR)$(UIDIR)/js

$(BUILDDIR):
	rm -rf $@ $@.tmp
	mkdir -p $@.tmp/ui
	cp -a debian/ src/ pwt-assets/ images/ pve.css index.html manifest.json index.html.tpl Makefile Cargo.toml $@.tmp/
	cp -a proxmox-api-types $@.tmp/
	echo "git clone git://git.proxmox.com/git/$(PACKAGE).git\\ngit checkout $$(git rev-parse HEAD)" \
	    > $@.tmp/debian/SOURCE
	mv $@.tmp $@

$(ORIG_SRC_TAR): $(BUILDDIR)
	tar czf $(ORIG_SRC_TAR) --exclude="$(BUILDDIR)/debian" $(BUILDDIR)

.PHONY: deb
deb: $(DEB)
$(DEB): $(BUILDDIR)
	cd $(BUILDDIR); dpkg-buildpackage -b -uc -us
	lintian $(DEB)
	@echo $(DEB)

.PHONY: dsc
dsc: $(BUILDDIR)
	rm -rf $(DSC) $(BUILDDIR)
	$(MAKE) $(DSC)
	lintian $(DSC)

$(DSC): $(BUILDDIR) $(ORIG_SRC_TAR)
	cd $(BUILDDIR)/ui; dpkg-buildpackage -S -us -uc -d
	dcmd mv $(BUILDDIR)/*.dsc ./

sbuild: $(DSC)
	sbuild $(DSC)

.PHONY: upload
upload: UPLOAD_DIST ?= $(DEB_DISTRIBUTION)
upload: $(DEB)
	tar cf - $(DEB) |ssh -X repoman@repo.proxmox.com -- upload --product pve --dist $(UPLOAD_DIST) --arch $(DEB_HOST_ARCH)

.PHONY: clean distclean
distclean: clean
clean:
	$(CARGO) clean
	rm -rf $(PACKAGE)-[0-9]*/ build/ dist/
	rm -f *.deb *.changes *.dsc *.tar.* *.buildinfo *.build .do-cargo-build

.PHONY: dinstall
dinstall: deb
	dpkg -i $(DEB)
