// Reenvía a Patina, por native messaging, dos cosas:
// - la selección hecha dentro de una página (desde content.js), y
// - la pestaña activa, para las capturas que no salen de una página, como la barra de direcciones.
// Ambas llevan el favicon de la página. El host las deja en $XDG_RUNTIME_DIR y guarda el favicon
// en ~/.local/share/patina/favicons/.

const HOST = "io.github.lautarosegura.patina";
const faviconCache = new Map(); // origen -> PNG en base64

// _favicon/ devuelve el ícono que Brave ya tiene en caché, sin pedirlo a la red.
async function favicon(pageUrl) {
  const origin = new URL(pageUrl).origin;
  if (!faviconCache.has(origin)) {
    const url = chrome.runtime.getURL(`/_favicon/?pageUrl=${encodeURIComponent(pageUrl)}&size=32`);
    const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
    faviconCache.set(origin, btoa(String.fromCharCode(...bytes)));
  }
  return faviconCache.get(origin);
}

async function send(payload) {
  const icon = await favicon(payload.url).catch(() => "");
  await chrome.runtime.sendNativeMessage(HOST, { ...payload, favicon: icon }).catch((error) => {
    console.warn("Patina no respondió; ¿está instalado el host nativo?", error);
  });
}

chrome.runtime.onMessage.addListener((payload, sender) => {
  if (sender.tab) send(payload); // solo mensajes de los content scripts
});

// Pestaña activa de la ventana con foco. tab.url y tab.title requieren el permiso "tabs"; las
// páginas internas como brave://settings se ignoran. El host anota desde cuándo está activa.
let lastPage = "";
async function reportActiveTab() {
  const [tab] = await chrome.tabs.query({ active: true, lastFocusedWindow: true });
  if (!tab?.url?.startsWith("http")) return;
  const key = `${tab.url}\n${tab.title}`;
  if (key === lastPage) return;
  lastPage = key;
  send({ kind: "page", tab: tab.id, url: tab.url, title: tab.title ?? "" });
}

chrome.tabs.onActivated.addListener(reportActiveTab);
chrome.windows.onFocusChanged.addListener(reportActiveTab);
chrome.tabs.onUpdated.addListener((_id, change, tab) => {
  if (tab.active && (change.url || change.title || change.status === "complete")) reportActiveTab();
});
