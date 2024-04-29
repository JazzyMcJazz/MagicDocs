import { EditorSelection } from '@codemirror/state';
import { Command, EditorView } from '@codemirror/view';

const toggleItalic: Command = (view) => {
    const changes = fromPattern("*", view);
    if (changes.changes.length) {
        view.dispatch(view.state.update(changes));
        return true;
    }
    return false;
};

const toggleBold: Command = (view) => {
    const changes = fromPattern("**", view);
    if (changes.changes.length) {
        view.dispatch?.(view.state.update(changes));
        return true;
    }
    return true;
};

export const customKeymap = [
    { key: "Mod-b", run: toggleBold},
    { key: "Mod-i", run: toggleItalic},
];

function fromPattern(pattern: string, { state }: EditorView) {
    return state.changeByRange(range => {
        const text = state.doc.sliceString(range.from, range.to);
        const isBold = text.startsWith(pattern) && text.endsWith(pattern);
        if (isBold) {
            return {
                changes: [
                    { from: range.from, to: range.from + 1 },
                    { from: range.to - 1, to: range.to }
                ],
                range: EditorSelection.range(range.from, range.to - 2)
            };
        } else {
            return {
                changes: [
                    { from: range.from, insert: pattern },
                    { from: range.to, insert: pattern }
                ],
                range: EditorSelection.range(range.from, range.to + 2)
            };
        }
    });
}