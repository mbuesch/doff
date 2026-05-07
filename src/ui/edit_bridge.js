/* edit_bridge.js - live editing support for döff
 *
 * Message format sent to Rust: "side\x01lineNum\x01text"
 *   side    = "left" | "right"
 *   lineNum = 1-based line number (decimal string)
 *   text    = plain-text content of the line (newlines stripped)
 */

(function () {
    'use strict';

    function syncContent(el) {
        if (el.dataset.content !== undefined && el !== document.activeElement) {
            el.textContent = el.dataset.content;
        }
    }

    /* Attribute observer: react to data-content changes. */
    var attrObs = new MutationObserver(function (muts) {
        for (var i = 0; i < muts.length; i++) {
            if (muts[i].attributeName === 'data-content') syncContent(muts[i].target);
        }
    });

    /* Return the current cursor column offset within a contenteditable element. */
    function getCursorOffset(el) {
        var sel = window.getSelection();
        if (!sel || sel.rangeCount === 0) return 0;
        var range = sel.getRangeAt(0);
        /* Measure from the start of el to the collapsed cursor position. */
        var preRange = document.createRange();
        preRange.selectNodeContents(el);
        preRange.setEnd(range.startContainer, range.startOffset);
        return preRange.toString().length;
    }

    /* Move focus to the prev/next .cell-input on the same side, preserving column. */
    function moveFocus(el, dir) {
        var col = getCursorOffset(el);
        var side = el.getAttribute('data-side');
        var cells = Array.prototype.slice.call(
            document.querySelectorAll('.cell-input[data-side="' + side + '"]')
        );
        var idx = cells.indexOf(el);
        var next = idx + dir;
        if (next < 0 || next >= cells.length) return;
        var target = cells[next];
        target.focus();
        /* Place cursor at same column, clamped to target length. */
        var sel = window.getSelection();
        if (!sel) return;
        var range = document.createRange();
        var textNode = target.firstChild;
        if (textNode && textNode.nodeType === Node.TEXT_NODE) {
            var pos = Math.min(col, textNode.length);
            range.setStart(textNode, pos);
            range.collapse(true);
        } else {
            range.selectNodeContents(target);
            range.collapse(false);
        }
        sel.removeAllRanges();
        sel.addRange(range);
    }

    /* Per-cell setup: sync content, watch attribute changes, block Enter/arrows. */
    function initCell(el) {
        syncContent(el);
        attrObs.observe(el, { attributes: true, attributeFilter: ['data-content'] });
        el.addEventListener('keydown', function (e) {
            if (e.key === 'Enter') {
                e.preventDefault();
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                moveFocus(el, -1);
            } else if (e.key === 'ArrowDown') {
                e.preventDefault();
                moveFocus(el, 1);
            }
        });
    }

    /* DOM observer: detect newly added .cell-input elements. */
    var domObs = new MutationObserver(function (muts) {
        for (var i = 0; i < muts.length; i++) {
            var added = muts[i].addedNodes;
            for (var j = 0; j < added.length; j++) {
                var node = added[j];
                if (node.nodeType !== 1) continue;
                if (node.classList && node.classList.contains('cell-input')) {
                    initCell(node);
                }
                if (node.querySelectorAll) {
                    var cells = node.querySelectorAll('.cell-input');
                    for (var k = 0; k < cells.length; k++) initCell(cells[k]);
                }
            }
        }
    });

    /* Clean up previous observer instances (guard against re-eval). */
    if (window._doffAttr) window._doffAttr.disconnect();
    if (window._doffDom)  window._doffDom.disconnect();
    window._doffAttr = attrObs;
    window._doffDom  = domObs;

    domObs.observe(document.body || document.documentElement,
                   { subtree: true, childList: true });

    /* Init cells that are already in the DOM (e.g. after a hot-reload). */
    var existing = document.querySelectorAll('.cell-input');
    for (var i = 0; i < existing.length; i++) initCell(existing[i]);

    /* Rust interface */
    function onInput(e) {
        var el = e.target;
        if (!el || !el.classList || !el.classList.contains('cell-input')) return;
        var lineNum = el.getAttribute('data-line-num');
        if (!lineNum) return;
        var side = el.getAttribute('data-side') || '';
        /* Strip stray newlines (paste guard; Enter is already blocked). */
        var text = el.textContent.replace(/\r?\n|\r/g, '');
        /* Protocol: side \x01 lineNum \x01 text */
        dioxus.send(side + '\x01' + lineNum + '\x01' + text);
    }

    if (window._doffInput) document.removeEventListener('input', window._doffInput, true);
    window._doffInput = onInput;
    document.addEventListener('input', window._doffInput, true);
})();
