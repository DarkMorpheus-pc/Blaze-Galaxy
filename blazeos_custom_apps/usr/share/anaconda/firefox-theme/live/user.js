/* BlazeOS local-only Anaconda browser profile. */
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
user_pref("layout.css.has-selector.enabled", true);
user_pref("browser.aboutConfig.showWarning", false);
user_pref("devtools.chrome.enabled", false);
user_pref("devtools.debugger.remote-enabled", false);
user_pref("browser.startup.page", 0);
user_pref("browser.startup.homepage", "about:blank");
user_pref("browser.startup.homepage_override.once", "");
user_pref("browser.toolbars.bookmarks.visibility", "never");
user_pref("browser.newtabpage.activity-stream.default.sites", "");
user_pref("browser.newtabpage.activity-stream.showSponsored", false);
user_pref("browser.newtabpage.activity-stream.showSponsoredTopSites", false);
user_pref("browser.newtabpage.activity-stream.feeds.telemetry", false);
user_pref("browser.newtabpage.activity-stream.telemetry", false);
user_pref("browser.ping-centre.telemetry", false);
user_pref("browser.uiCustomization.state", "{\"placements\":{\"widget-overflow-fixed-list\":[],\"unified-extensions-area\":[],\"nav-bar\":[\"back-button\",\"forward-button\",\"stop-reload-button\",\"urlbar-container\"],\"toolbar-menubar\":[\"menubar-items\"],\"TabsToolbar\":[\"tabbrowser-tabs\"],\"PersonalToolbar\":[\"personal-bookmarks\"]},\"seen\":[],\"dirtyAreaCache\":[\"nav-bar\",\"TabsToolbar\"],\"currentVersion\":19,\"newElementCount\":0}");
user_pref("browser.urlbar.placeholderName", "");

/* The installer loads only 127.0.0.1. Keep its temporary browser offline. */
user_pref("app.normandy.enabled", false);
user_pref("app.shield.optoutstudies.enabled", false);
user_pref("browser.discovery.enabled", false);
user_pref("browser.region.network.url", "");
user_pref("browser.region.update.enabled", false);
user_pref("browser.search.update", false);
user_pref("browser.safebrowsing.downloads.enabled", false);
user_pref("browser.safebrowsing.malware.enabled", false);
user_pref("browser.safebrowsing.phishing.enabled", false);
user_pref("browser.safebrowsing.provider.google.updateURL", "");
user_pref("browser.safebrowsing.provider.google4.updateURL", "");
user_pref("datareporting.healthreport.uploadEnabled", false);
user_pref("datareporting.policy.dataSubmissionEnabled", false);
user_pref("dom.push.enabled", false);
user_pref("extensions.getAddons.cache.enabled", false);
user_pref("extensions.systemAddon.update.enabled", false);
user_pref("extensions.update.enabled", false);
user_pref("geo.provider.network.url", "");
user_pref("network.captive-portal-service.enabled", false);
user_pref("network.connectivity-service.enabled", false);
user_pref("network.trr.mode", 5);
user_pref("services.settings.server", "");
user_pref("toolkit.telemetry.archive.enabled", false);
user_pref("toolkit.telemetry.enabled", false);
user_pref("toolkit.telemetry.server", "");
user_pref("toolkit.telemetry.unified", false);
user_pref("trailhead.firstrun.didSeeAboutWelcome", true);

/*
 * Firefox keeps adding background services whose individual preferences can
 * change between releases.  The Anaconda UI is entirely local, so route all
 * non-local HTTP(S) requests to a closed loopback port.  localhost remains
 * direct and the installer keeps working without leaking background traffic.
 */
user_pref("network.proxy.type", 1);
user_pref("network.proxy.share_proxy_settings", true);
user_pref("network.proxy.http", "127.0.0.1");
user_pref("network.proxy.http_port", 9);
user_pref("network.proxy.ssl", "127.0.0.1");
user_pref("network.proxy.ssl_port", 9);
user_pref("network.proxy.no_proxies_on", "localhost, 127.0.0.1, ::1");

user_pref("network.protocol-handler.warn-external-default", false);
user_pref("network.protocol-handler.external.anaconda-gnome-control-center", true);
user_pref("network.protocol-handler.external.extlink", true);
user_pref("dom.disable_open_during_load", false);
user_pref("signon.showAutoCompleteFooter", false);
user_pref("signon.rememberSignons", false);
