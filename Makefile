include /usr/share/dpkg/default.mk

PACKAGE=pve-yew-mobile-gui
CRATENAME=pve-yew-mobile-gui

BUILDDIR ?= $(PACKAGE)-$(DEB_VERSION_UPSTREAM)
ORIG_SRC_TAR=$(PACKAGE)_$(DEB_VERSION_UPSTREAM).orig.tar.gz

DEB=$(PACKAGE)_$(DEB_VERSION)_$(DEB_HOST_ARCH).deb
DBG_DEB=$(PACKAGE)-dbgsym_$(DEB_VERSION)_$(DEB_HOST_ARCH).deb
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
UIDIR = $(PREFIX)/share/javascript/pve-yew-mobile-gui

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

dist/material-yew-style.css: pwt-assets/scss/material-yew-style.scss
	rust-grass $< $@

install: $(COMPILED_OUTPUT) index.html manifest.json dist/material-yew-style.css
	install -dm0755 $(DESTDIR)$(UIDIR)
	install -dm0755 $(DESTDIR)$(UIDIR)/js

	install -dm0755 $(DESTDIR)$(UIDIR)/images
	#install -m0644 images/favicon.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/icon-cpu.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/icon-memory.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo_icon_black.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo_icon_white.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox-icon-512.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox-icon-192.png $(DESTDIR)$(UIDIR)/images
	install -m0644 images/screenshot-dashboard.png $(DESTDIR)$(UIDIR)/images

	install -dm0755 $(DESTDIR)$(UIDIR)/fonts
	install -m0644 pwt-assets/assets/font-awesome.css $(DESTDIR)$(UIDIR)/fonts
	install -m0644 pwt-assets/assets/fonts/RobotoFlexVariableFont.ttf $(DESTDIR)$(UIDIR)/fonts

	install -m0644 dist/pve-yew-mobile-gui_bundle.js $(DESTDIR)$(UIDIR)/js
	install -m0644 dist/pve-yew-mobile-gui_bg.wasm.gz $(DESTDIR)$(UIDIR)/js
	install -m0644 index.html $(DESTDIR)$(UIDIR)
	install -m0644 manifest.json $(DESTDIR)$(UIDIR)
	install -m0644 dist/material-yew-style.css $(DESTDIR)$(UIDIR)


$(BUILDDIR):
	rm -rf $@ $@.tmp
	mkdir -p $@.tmp/ui
	cp -a debian/ src/ pwt-assets/ assets/ pve.css index.html Makefile Cargo.toml $@.tmp/
	cp -a proxmox-api-types $@.tmp/
	echo "git clone git://git.proxmox.com/git/$(PACKAGE).git\\ngit checkout $$(git rev-parse HEAD)" \
	    > $@.tmp/debian/SOURCE
	mv $@.tmp $@

$(ORIG_SRC_TAR): $(BUILDDIR)
	tar czf $(ORIG_SRC_TAR) --exclude="$(BUILDDIR)/debian" $(BUILDDIR)

.PHONY: deb
deb: $(DEB)
$(DBG_DEB): $(DEB)
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
upload: $(DEB) $(DBG_DEB)
	tar cf - $(DEB) $(DBG_DEB) |ssh -X repoman@repo.proxmox.com -- upload --product pdm --dist $(UPLOAD_DIST) --arch $(DEB_HOST_ARCH)

.PHONY: clean distclean
distclean: clean
clean:
	$(CARGO) clean
	rm -rf $(PACKAGE)-[0-9]*/ build/ dist/
	rm -f *.deb *.changes *.dsc *.tar.* *.buildinfo *.build .do-cargo-build

.PHONY: dinstall
dinstall: deb
	dpkg -i $(DEB)
