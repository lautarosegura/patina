// Cuando resaltás texto, manda a Patina el texto, la URL y el título de la página. En WhatsApp Web
// agrega el remitente, la hora y el chat de los mensajes seleccionados. Patina usa estos datos
// solo si el texto coincide con lo que captura con doble Shift.
//
// WhatsApp marca cada mensaje con data-pre-plain-text="[20:15, 13/10/2026] Juan Pérez: ".
// Si WhatsApp cambia ese atributo, la captura sigue funcionando pero sin remitente.

const MESSAGE = "[data-pre-plain-text]";
const PRE_TEXT = /^\[([^\]]+)\]\s*(.*?):\s*$/;
const isWhatsApp = location.origin === "https://web.whatsapp.com";
let timer = 0;
let lastSent = "";

document.addEventListener("selectionchange", () => {
  clearTimeout(timer);
  timer = setTimeout(report, 300); // esperar a que termine el arrastre del mouse
});

function selectedMessages(range) {
  const common = range.commonAncestorContainer;
  const element = common.nodeType === Node.ELEMENT_NODE ? common : common.parentElement;
  const enclosing = element?.closest(MESSAGE);
  if (enclosing) return [enclosing];
  return [...(element?.querySelectorAll(MESSAGE) ?? [])].filter((el) => range.intersectsNode(el));
}

// WhatsApp escribe la fecha según el idioma del navegador ("10/13/2026" en inglés de EE. UU.).
// Patina necesita el orden para no confundir día y mes en fechas como 3/4.
const dateOrder = new Intl.DateTimeFormat(navigator.language)
  .formatToParts(new Date(2000, 11, 31))
  .filter((part) => ["day", "month", "year"].includes(part.type))
  .map((part) => part.type[0])
  .join("");

function chatName() {
  const title = document.querySelector("#main header span[dir='auto']");
  return title?.textContent.trim() || "";
}

function whatsappDetails(range) {
  const messages = selectedMessages(range)
    .map((el) => el.getAttribute("data-pre-plain-text").match(PRE_TEXT))
    .filter(Boolean)
    .map(([, time, sender]) => ({ time, sender }));
  return messages.length ? { messages, chat: chatName(), dateOrder } : {};
}

// Texto de la selección tal como se ve. Selection.toString() (igual que la selección primaria de
// Chromium) corta la línea alrededor de elementos inline-flex o inline-block, como las menciones
// de X: "asking @bot:" sale "asking \n@bot\n:". Acá solo cortan los bloques, <br> y los saltos
// del propio texto en elementos con white-space pre*. Patina lo usa únicamente si tiene los mismos
// caracteres que la selección del sistema, salvo espacios.
const MAX_NODES = 20000;

function visibleText(range) {
  const root = range.commonAncestorContainer;
  const start = root.nodeType === Node.ELEMENT_NODE ? root : root.parentElement;
  const parts = []; // strings y cortes {lines: n}
  let visited = 0;

  function isBlock(display) {
    return !display.startsWith("inline") && display !== "contents" && display !== "none";
  }

  // inline: estamos dentro de un elemento inline-flex/inline-grid/inline-block. Sus hijos figuran
  // como display: block (los ítems flex se "bloquifican"), pero en pantalla siguen en la línea.
  function walk(node, inline = false) {
    if (++visited > MAX_NODES) throw new Error("selección demasiado grande");
    if (!range.intersectsNode(node)) return;
    if (node.nodeType === Node.TEXT_NODE) {
      let text = node.data;
      if (node === range.endContainer) text = text.slice(0, range.endOffset);
      if (node === range.startContainer) text = text.slice(range.startOffset);
      const ws = node.parentElement ? getComputedStyle(node.parentElement).whiteSpace : "normal";
      if (!ws.startsWith("pre") && ws !== "break-spaces") text = text.replace(/\s+/g, " ");
      else if (ws === "pre-line") text = text.replace(/[^\S\n]+/g, " ");
      parts.push(text);
      return;
    }
    if (node.nodeType !== Node.ELEMENT_NODE) return;
    if (node.tagName === "BR") return void parts.push({ lines: 1 });
    const style = getComputedStyle(node);
    if (style.display === "none" || style.visibility === "hidden") return;
    const block = node !== start && !inline && isBlock(style.display);
    const lines = node.tagName === "P" ? 2 : 1;
    const childInline = inline || (style.display.startsWith("inline") && style.display !== "inline");
    if (block) parts.push({ lines });
    for (const child of node.childNodes) walk(child, childInline);
    if (block) parts.push({ lines });
  }

  walk(start);
  let out = "";
  let pending = 0;
  for (const part of parts) {
    if (typeof part !== "string") pending = Math.max(pending, part.lines);
    else if (part) {
      if (pending && out) out = out.replace(/ +$/, "") + "\n".repeat(pending);
      out += pending && out.endsWith("\n") ? part.replace(/^ +/, "") : part;
      pending = 0;
    }
  }
  return out.trim();
}

// La selección como HTML mínimo (<b>, <i>, <code>, <a>) según los estilos calculados, así cuenta
// también la negrita que la página pone por CSS. Patina toma de acá solo el formato: el texto sale
// de visibleText.
const MONOSPACE = /mono|courier|consolas|menlo/i;

function escapeHtml(text) {
  return text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

// Como visibleText, que no entra en elementos ocultos: un texto bajo un ancestro oculto no se ve.
function hiddenChecker() {
  const hidden = new Map();
  return function isHidden(element) {
    if (!element) return false;
    if (!hidden.has(element)) {
      const style = getComputedStyle(element);
      hidden.set(element, style.display === "none" || style.visibility === "hidden" || isHidden(element.parentElement));
    }
    return hidden.get(element);
  };
}

function formattedHtml(range) {
  const isHidden = hiddenChecker();
  const root = range.commonAncestorContainer;
  const walker = document.createTreeWalker(root.nodeType === Node.TEXT_NODE ? root.parentNode : root, NodeFilter.SHOW_TEXT);
  let html = "";
  let visited = 0;
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    if (++visited > MAX_NODES) throw new Error("selección demasiado grande");
    if (!range.intersectsNode(node) || !node.parentElement) continue;
    let text = node.data;
    if (node === range.endContainer) text = text.slice(0, range.endOffset);
    if (node === range.startContainer) text = text.slice(range.startOffset);
    if (!text.trim()) continue;
    if (isHidden(node.parentElement)) continue;
    const style = getComputedStyle(node.parentElement);
    let part = escapeHtml(text);
    if (MONOSPACE.test(style.fontFamily)) part = `<code>${part}</code>`;
    if (Number(style.fontWeight) >= 600) part = `<b>${part}</b>`;
    if (style.fontStyle === "italic") part = `<i>${part}</i>`;
    const href = node.parentElement.closest("a[href]")?.href;
    if (href?.startsWith("http")) part = `<a href="${escapeHtml(href)}">${part}</a>`;
    html += part;
  }
  return html;
}

function report() {
  const selection = document.getSelection();
  let text = selection?.rangeCount ? selection.toString() : "";
  if (text.trim()) {
    try {
      text = visibleText(selection.getRangeAt(0)) || text;
    } catch {
      // selección enorme o DOM raro: queda el texto de Chromium
    }
  }
  // También se avisa la selección vacía: Patina la usa para no capturar una selección que ya no se ve.
  const payload = { text: text.trim() ? text : "", url: location.href, title: document.title };
  if (payload.text) {
    try {
      payload.html = formattedHtml(selection.getRangeAt(0));
    } catch {
      // sin formato: queda el texto plano
    }
  }
  if (payload.text && isWhatsApp) Object.assign(payload, whatsappDetails(selection.getRangeAt(0)));
  const key = JSON.stringify(payload);
  if (key === lastSent) return;
  lastSent = key;
  chrome.runtime.sendMessage(payload).catch(() => {}); // el service worker se reinicia solo
}
