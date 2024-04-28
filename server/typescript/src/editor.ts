
function load() {
    new SimpleMDE({
        element: document.getElementById("editor") || undefined,
        toolbar: ['bold', 'italic', 'heading', '|', 'unordered-list', 'ordered-list', '|', 'link', 'image', '|', 'preview'],
        tabSize: 4,
        status: false,
        spellChecker: false,
        forceSync: true,
        shortcuts: {
            toggleFullScreen: '',
            toggleSideBySide: '',
            togglePreview: '',
        },
    });
}

htmx.onLoad(load);