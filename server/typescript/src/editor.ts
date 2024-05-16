import { EditorState } from "@codemirror/state";
import { EditorView, drawSelection, dropCursor, keymap } from "@codemirror/view";
import { markdown, markdownKeymap } from '@codemirror/lang-markdown';
import { closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { searchKeymap } from "@codemirror/search";
import { foldKeymap, indentOnInput } from "@codemirror/language";
import { lintKeymap } from "@codemirror/lint";
import { customKeymap } from "./lib/keybinds";

const parent = document.getElementById("editor") as HTMLDivElement;
const textarea = document.getElementById("textarea") as HTMLTextAreaElement;

const changeListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
        textarea.value = update.state.doc.toString();
    }
});

const extensions = [
    history(),
    drawSelection(),
    dropCursor(),
    indentOnInput(),
    changeListener,
    keymap.of([
        indentWithTab,
        ...customKeymap,
        ...closeBracketsKeymap,
        ...defaultKeymap,
        ...searchKeymap,
        ...historyKeymap,
        ...foldKeymap,
        ...completionKeymap,
        ...lintKeymap,
        ...markdownKeymap,
    ]),
    markdown(),
];



let didLoad = false;
function load() {
    if (!didLoad) {
        const state = EditorState.create({
            doc: textarea.value,
            extensions,
        });
        new EditorView({ parent, state });
        didLoad = true;
    }
}

htmx.onLoad(load);
load();
