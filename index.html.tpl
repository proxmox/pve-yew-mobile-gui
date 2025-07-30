<!--- Index template for pveproxy server -->
[%
    USE date;
    base_path = yew_mobile_base_path or '/pve2/yew-mobile';
    ui_version = yew_mobile_mtime or date.now;
    i18n_version = i18n_yew_mobile_mtime or date.now;
%]
<!DOCTYPE html>
<html>
<head>
  <meta http-equiv="Content-type" content="text/html; charset=utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">

  <title>[% nodename %] - Proxmox Virtual Environment</title>

  <link rel="manifest" href="[% base_path %]/manifest.json" />

  <link rel="stylesheet" type="text/css" href="[% base_path %]/css/font-awesome.css?v=[% ui_version %]" />
  <link rel="stylesheet" type="text/css" href="[% base_path %]/css/pve.css?v=[% ui_version %]" />

  <style>
    /* Avoid flickering (default background in firefox is always white)*/
    @media (prefers-color-scheme: dark) {
      body { background: #333; }
    }
    @media (prefers-color-scheme: light) {
      body { background: #fff; }
    }
  </style>
  <script type="text/javascript">
    // remove below, it's unused?
    Proxmox = {
        Setup: { auth_cookie_name: 'PVEAuthCookie' },
        defaultLang: '[% lang %]',
        NodeName: '[% nodename %]',
        UserName: '[% username %]',
        CSRFPreventionToken: '[% token %]',
        ConsentText: '[% consenttext %]',
        i18nVersion: [% i18n_version %],
        baseBath: [% base_path %],
    };
  </script>

  <link rel="preload" href="[% base_path %]/js/pve-yew-mobile-gui_bg.wasm?v=[% ui_version %]" as="fetch" type="application/wasm" crossorigin="">
  <link rel="modulepreload" href="[% base_path %]/js/pve-yew-mobile-gui_bundle.js?v=[% ui_version %]">

</head>

<body>
  <script type="module">
    import init from "[% base_path %]/js/pve-yew-mobile-gui_bundle.js?v=[% ui_version %]";
    const decompressedResponse = new Response(
      await fetch('[% base_path %]/js/pve-yew-mobile-gui_bg.wasm?v=[% ui_version %]').then(res => res.body)
    );
    // set correct type to allow using faster WebAssembly.instantiateStreaming
    decompressedResponse.headers.set("Content-Type", "application/wasm");
    init(decompressedResponse);
  </script>
</body>

</html>
